use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, PromiseOrValue};
mod constants;

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
        for (account_id, steps) in steps_batch.into_iter() {
            self.record(&account_id, steps);
        }
    }

    fn record(&mut self, account_id: &AccountId, steps: u32) -> U128 {
        if !self.token.accounts.contains_key(account_id) {
            self.token.internal_register_account(account_id);
        }
        let capped_steps = self.get_capped_steps(account_id, steps);
        let swt = self.formula(self.steps_from_tge, capped_steps);
        self.token.internal_deposit(account_id, swt.0 as u128);
        self.steps_from_tge.0 += capped_steps as u64;
        swt
    }

    pub fn formula(&self, steps_from_tge: U64, steps: u32) -> U128 {
        let trl = (steps_from_tge.0 as f64/ 1e+12).floor() as usize;
        if trl < 400 {
            let trl2 = ((steps_from_tge.0 as f64 + steps as f64) / 1e+12).floor() as usize;
            if trl != trl2 {
                // two lines
                let middle = (trl2 as f64) * 1e+12;
                let swt1 = constants::calculate_square_under_line(constants::KS[trl], constants::BS[trl], steps_from_tge.0 as f64, middle);
                let swt2 = constants::calculate_square_under_line(constants::KS[trl2], constants::BS[trl2], middle, middle + steps as f64);
                return U128(((swt1 + swt2) * constants::DECIMALS) as u128);
            } else {
                // one line 
                let swt = constants::calculate_square_under_line(constants::KS[trl], constants::BS[trl], steps_from_tge.0 as f64, steps_from_tge.0 as f64 + steps as f64);
                return U128((swt * constants::DECIMALS) as u128);
            }
        } else {
            let a: f64 = constants::DECIMALS * (constants::K.powf(steps as f64 + steps_from_tge.0 as f64 + 1.));
            let b: f64 = constants::DECIMALS * (constants::K.powf(steps_from_tge.0 as f64 + 1.));
            let swt: f64 = (a - b) / (constants::K - 1.) / 1000.;
            return U128(swt as u128)
        }        
    }

    fn get_capped_steps(&mut self, account_id: &AccountId, steps_to_convert: u32) -> u32 {
        let (mut sum, mut ts) = self.daily_limits.get(account_id).unwrap_or((0, 0));
        let current_ts: u64 = env::block_timestamp();
        let mut remaining_steps = 2 * constants::DAILY_STEP_CONVERSION_LIMIT;
        if ts == 0 || current_ts - ts >= constants::DAY_IN_NANOS {
            ts = current_ts;
            sum = 0;
        }

        // TODO can either variable cross u32 bounds? Cast will overflow
        remaining_steps = i32::max(0, remaining_steps as i32 - sum as i32) as u32;
        let capped_steps: u32 = u32::min(remaining_steps, steps_to_convert);
        self.daily_limits
            .insert(account_id, &(sum + capped_steps, ts));
        // println!("time = {}, remaining_steps = {}, steps_to_convert = {}, sum = {}", current_ts, remaining_steps, steps_to_convert, sum);
        capped_steps
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0".to_string(),
            name: "SWT (v0.3)".to_string(),
            symbol: "SWT (v0.3)".to_string(),
            icon: Some(String::from(constants::ICON)),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}

// :TODO: sandbox tests?
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::VMContext;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new().is_view(is_view).build()
    }

    #[test]
    fn test_daily_cap1() {
        let oracles = vec!["intmainreturn0.testnet".parse().unwrap()];
        let mut contract = Contract::new(oracles);
        assert_eq!(U64(0), contract.get_steps_from_tge());
        let alice: AccountId = "alice.testnet".parse().unwrap();
        
        // constants::BS.len();
        // for i in 0..constants::BS.len(){
        //     print!("{}, ", constants::BS[i]);
        // }
        let steps_to_convert = vec!(1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000);
        let res = vec!(0.0, 0.008999999999991565, 0.09899999999914803, 0.998999999914795, 9.998999991479492, 99.99899914794923, 999.9989147949235, 9999.990479492355, 99999.14694923519);
        
        for i in 0..steps_to_convert.len() - 1 {
            let diff = contract.formula(U64(1), steps_to_convert[i]).0 as f64 / constants::DECIMALS - res[i];
            println!("{} {} {}", res[i], contract.formula(U64(1), steps_to_convert[i]).0 as f64 / constants::DECIMALS, diff.abs());
        }
    }

}
