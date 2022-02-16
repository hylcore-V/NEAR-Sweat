use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::json_types::{U128, U64};
mod constants;
mod math;

use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    oracles: LookupSet<AccountId>,
    token: FungibleToken,
    steps_from_tge: U64,
    daily_limits: LookupMap<AccountId, (u32, u64)>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(oracles_vec: Vec<AccountId>) -> Self {
        let mut oracles_tree = LookupSet::new(b"s");
        for oracle in oracles_vec.iter() {
            env::log_str(oracle.as_str());
            oracles_tree.insert(oracle);
        }
        Self {
            oracles: oracles_tree,
            token: FungibleToken::new(b"t"),
            steps_from_tge: U64::from(0),
            daily_limits: LookupMap::new(b"l"),
        }
    }

    pub fn get_steps_from_tge(&self) -> U64 {
        self.steps_from_tge
    }

    pub fn record_batch(&mut self, steps_batch: Vec<(AccountId, u32)>) {
        assert!(self.oracles.contains(&env::predecessor_account_id()));
        let mut oracle_fee: u128 = 0;
        for (account_id, steps) in steps_batch.into_iter() {
            let capped_steps = self.get_capped_steps(&account_id, steps);
            let sweat_to_mint: u128 = self.formula(self.steps_from_tge, capped_steps).0;
            let trx_oracle_fee: u128 = sweat_to_mint * 5 / 100;
            let minted_to_user: u128 = sweat_to_mint - trx_oracle_fee;
            oracle_fee = oracle_fee + trx_oracle_fee;
            internal_deposit(&mut self.token, &account_id, minted_to_user);
            self.steps_from_tge.0 += capped_steps as u64;
        }
        internal_deposit(&mut self.token, &env::predecessor_account_id(), oracle_fee);
    }

    pub fn formula(&self, steps_from_tge: U64, steps: u32) -> U128 {
        U128(math::formula(steps_from_tge.0 as f64, steps as f64))
    }

    fn get_capped_steps(&mut self, account_id: &AccountId, steps_to_convert: u32) -> u32 {
        let (mut sum, mut ts) = self.daily_limits.get(account_id).unwrap_or((0, 0));
        let current_ts: u64 = env::block_timestamp();
        const DAY_IN_NANOS: u64 = 86_400_000_000_000;
        const DAILY_STEP_CONVERSION_LIMIT: u32 = 10_000;
        let mut remaining_steps = 2 * DAILY_STEP_CONVERSION_LIMIT;
        if ts == 0 || current_ts - ts >= DAY_IN_NANOS {
            ts = current_ts;
            sum = 0;
        }

        // TODO can either variable cross u32 bounds? Cast will overflow
        remaining_steps = i32::max(0, remaining_steps as i32 - sum as i32) as u32;
        let capped_steps: u32 = u32::min(remaining_steps, steps_to_convert);
        self.daily_limits
            .insert(account_id, &(sum + capped_steps, ts));
        capped_steps
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

/// Taken from contract standards but modified to default if account isn't initialized
/// rather than panicking:
/// <https://github.com/near/near-sdk-rs/blob/6596dc311036fe51d94358ac8f6497ef6e5a7cfc/near-contract-standards/src/fungible_token/core_impl.rs#L105>
fn internal_deposit(token: &mut FungibleToken, account_id: &AccountId, amount: Balance) {
    let balance = token.accounts.get(account_id).unwrap_or_default();
    let new_balance = balance
        .checked_add(amount)
        .unwrap_or_else(|| env::panic_str("Balance overflow"));
    token.accounts.insert(account_id, &new_balance);
    token.total_supply = token
        .total_supply
        .checked_add(amount)
        .unwrap_or_else(|| env::panic_str("Total supply overflow"));
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0".to_string(),
            name: "SWEAT".to_string(),
            symbol: "SWEAT".to_string(),
            icon: Some(String::from(constants::ICON)),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}

// :TODO: workspaces tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    const EPS: f64 = 0.00001;
    use near_sdk::testing_env;
    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id)
            .attached_deposit(1);
        builder
    }

    #[test]
    fn oracle_fee_test() {
        let context = get_context(accounts(0));
        testing_env!(context.build());
        let oracles = vec![accounts(0)];
        let mut contract = Contract::new(oracles);
        assert_eq!(U64(0), contract.get_steps_from_tge());
        contract.record_batch(vec![(accounts(1), 10_000), (accounts(2), 10_000)]);
        assert_eq!(
            true,
            (1. - contract.token.ft_balance_of(accounts(0)).0 as f64 / 1e+18).abs() < EPS
        );
        assert_eq!(
            true,
            (9.5 - contract.token.ft_balance_of(accounts(1)).0 as f64 / 1e+18).abs() < EPS
        );
        assert_eq!(
            true,
            (9.5 - contract.token.ft_balance_of(accounts(2)).0 as f64 / 1e+18).abs() < EPS
        );
        assert_eq!(U64(2 * 10_000), contract.get_steps_from_tge());
    }

    #[test]
    fn formula_test() {
        let oracles = vec!["intmainreturn0.testnet".parse().unwrap()];
        let contract = Contract::new(oracles);
        assert_eq!(U64(0), contract.get_steps_from_tge());

        let steps_to_convert = vec![
            1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000,
        ];
        let steps_from_tge = vec![
            1,
            10,
            100,
            1000,
            10000,
            100000,
            1000000,
            10000000,
            100000000,
            1000000000,
            10000000000,
            100000000000,
            1000000000000,
            10000000000000,
            100000000000000,
            1000000000000000u64,
            999999999000,
        ];
        let mut test_number = 0;
        for tge in 0..steps_from_tge.len() {
            for steps in 0..steps_to_convert.len() {
                let formula_res = contract
                    .formula(U64(steps_from_tge[tge]), steps_to_convert[steps])
                    .0 as f64
                    / 1e+18;
                let diff = formula_res - TEST_RESULTS[test_number];
                assert_eq!(true, diff.abs() < EPS);
                test_number = test_number + 1;
            }
        }
    }

    pub const TEST_RESULTS: [f64; 153] = [
        0.0009999999999997387,
        0.009999999999989545,
        0.09999999999911131,
        0.9999999999126997,
        9.999999991285653,
        99.99999912872224,
        999.9999128737927,
        9999.991287394952,
        99999.12873965199,
        0.0009999999999981703,
        0.009999999999973861,
        0.09999999999895448,
        0.9999999999111314,
        9.999999991269972,
        99.99999912856542,
        999.9999128722244,
        9999.99128737927,
        99999.12873949517,
        0.0009999999999824878,
        0.009999999999817035,
        0.09999999999738622,
        0.9999999998954487,
        9.999999991113144,
        99.99999912699715,
        999.9999128565418,
        9999.991287222441,
        99999.1287379269,
        0.0009999999998256607,
        0.009999999998248767,
        0.09999999998170353,
        0.9999999997386219,
        9.999999989544875,
        99.99999911131447,
        999.9999126997149,
        9999.991285654174,
        99999.12872224422,
        0.0009999999982573922,
        0.009999999982566081,
        0.09999999982487667,
        0.9999999981703533,
        9.99999997386219,
        99.99999895448761,
        999.9999111314463,
        9999.991269971488,
        99999.12856541736,
        0.0009999999825747062,
        0.00999999982573922,
        0.09999999825660807,
        0.9999999824876673,
        9.99999981703533,
        99.99999738621901,
        999.9998954487603,
        9999.991113144628,
        99999.12699714876,
        0.0009999998257478467,
        0.009999998257470626,
        0.09999998257392213,
        0.9999998256608078,
        9.999998248766735,
        99.99998170353305,
        999.9997386219009,
        9999.989544876033,
        99999.1113144628,
        0.0009999982574792517,
        0.009999982574784676,
        0.09999982574706262,
        0.9999982573922128,
        9.999982566080785,
        99.99982487667356,
        999.9981703533058,
        9999.973862190083,
        99998.9544876033,
        0.0009999825747933012,
        0.009999825747925172,
        0.09999825747846758,
        0.9999825747062624,
        9.999825739221281,
        99.9982566080785,
        999.9824876673554,
        9999.817035330578,
        99997.38621900826,
        0.0009998257479337971,
        0.009998257479330131,
        0.09998257479251717,
        0.9998257478467583,
        9.998257470626239,
        99.9825739221281,
        999.8256608078512,
        9998.248766735538,
        99981.70353305785,
        0.0009982574793387558,
        0.009982574793379717,
        0.09982574793301303,
        0.9982574792517169,
        9.982574784675826,
        99.82574706262396,
        998.2573922128099,
        9982.566080785124,
        99824.87667355372,
        0.0009825747933883426,
        0.009825747933875585,
        0.09825747933797171,
        0.9825747933013037,
        9.825747925171694,
        98.25747846758264,
        982.5747062623967,
        9825.739221280992,
        98256.60807851239,
        0.0008257479338842365,
        0.00825747933883687,
        0.08257479338781916,
        0.8257479338232387,
        8.257479332737093,
        82.5747927778415,
        825.7478728254714,
        8257.473232960365,
        82574.18280016878,
        0.00032230806451611884,
        0.003223080645160268,
        0.03223080645151066,
        0.32230806450590477,
        3.223080644138863,
        32.23080634937016,
        322.3080542918551,
        3223.0796227338956,
        32230.704208873405,
        4.541613636363615e-05,
        0.0004541613636363422,
        0.0045416136363614894,
        0.04541613636342166,
        0.45416136361489273,
        4.541613634216543,
        45.41613614892703,
        454.1613421654302,
        4541.611489270292,
        4.741577501003273e-06,
        4.739512354008062e-05,
        0.00047393471422484454,
        0.004739338881660464,
        0.047393368165134696,
        0.473933648608995,
        4.739336490220244,
        47.393364695687744,
        473.9336257065148,
        0.0008257479340584625,
        0.008257479340576784,
        0.08257479340498369,
        0.8257479339714235,
        8.257479333984337,
        82.57479279007933,
        825.7478729476152,
        8257.473234181569,
        82574.18281238058,
    ];
}
