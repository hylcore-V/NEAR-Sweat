use near_contract_standards::fungible_token::events::FtBurn;
use near_contract_standards::fungible_token::events::FtMint;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::json_types::{U128, U64};
mod math;

use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    oracles: LookupSet<AccountId>,
    token: FungibleToken,
    steps_from_tge: U64,
    daily_limits: LookupMap<AccountId, (u16, u64)>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            oracles: LookupSet::new(b"s"),
            token: FungibleToken::new(b"t"),
            steps_from_tge: U64::from(0),
            daily_limits: LookupMap::new(b"l"),
        }
    }
    #[private]
    pub fn add_oracle(&mut self, account_id: &AccountId) {
        assert_eq!(env::predecessor_account_id(), env::current_account_id());
        self.oracles.insert(account_id);
    }

    #[private]
    pub fn remove_oracle(&mut self, account_id: &AccountId) {
        assert_eq!(env::predecessor_account_id(), env::current_account_id());
        self.oracles.remove(account_id);
    }

    pub fn mint_tge(&mut self, amount: U128, account_for: AccountId) {
        assert!(self.oracles.contains(&env::predecessor_account_id()));
        self.token.internal_register_account(&account_for);
        internal_deposit(&mut self.token, &env::predecessor_account_id(), amount.0);
    }

    pub fn burn(&mut self, amount: &U128) {
        self.token
            .internal_withdraw(&env::predecessor_account_id(), amount.0);
        FtBurn {
            owner_id: &env::predecessor_account_id(),
            amount: amount,
            memo: None,
        }
        .emit()
    }

    pub fn get_steps_from_tge(&self) -> U64 {
        self.steps_from_tge
    }

    pub fn record_batch(&mut self, steps_batch: Vec<(AccountId, u16)>) {
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

    pub fn formula(&self, steps_from_tge: U64, steps: u16) -> U128 {
        U128(math::formula(steps_from_tge.0 as f64, steps as f64))
    }

    fn get_capped_steps(&mut self, account_id: &AccountId, steps_to_convert: u16) -> u16 {
        let (mut sum, mut ts) = self.daily_limits.get(account_id).unwrap_or((0, 0));
        let current_ts: u64 = env::block_timestamp();
        const DAY_IN_NANOS: u64 = 86_400_000_000_000;
        const DAILY_STEP_CONVERSION_LIMIT: u16 = 10_000;
        let mut remaining_steps = 2 * DAILY_STEP_CONVERSION_LIMIT;
        if ts == 0 || current_ts - ts >= DAY_IN_NANOS {
            ts = current_ts;
            sum = 0;
        }

        remaining_steps = i16::max(0, remaining_steps as i16 - sum as i16) as u16;
        let capped_steps: u16 = u16::min(remaining_steps, steps_to_convert);
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
    FtMint {
        owner_id: account_id,
        amount: &U128(amount),
        memo: None,
    }
    .emit()
}

pub const ICON: &str = "data:image/svg+xml,%3Csvg viewBox='0 0 100 100' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Crect width='100' height='100' rx='50' fill='%23FF0D75'/%3E%3Cg clip-path='url(%23clip0_283_2788)'%3E%3Cpath d='M39.4653 77.5455L19.0089 40.02L35.5411 22.2805L55.9975 59.806L39.4653 77.5455Z' stroke='white' stroke-width='10'/%3E%3Cpath d='M66.0253 77.8531L45.569 40.3276L62.1012 22.5882L82.5576 60.1136L66.0253 77.8531Z' stroke='white' stroke-width='10'/%3E%3C/g%3E%3Cdefs%3E%3CclipPath id='clip0_283_2788'%3E%3Crect width='100' height='56' fill='white' transform='translate(0 22)'/%3E%3C/clipPath%3E%3C/defs%3E%3C/svg%3E%0A";

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0".to_string(),
            name: "SWEAT".to_string(),
            symbol: "SWEAT".to_string(),
            icon: Some(String::from(ICON)),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}
