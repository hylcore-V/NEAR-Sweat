use near_contract_standards::fungible_token::events::{FtBurn, FtMint};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::{U128, U64};
use near_sdk::require;
mod math;

use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[macro_use]
extern crate static_assertions;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    oracles: UnorderedSet<AccountId>,
    token: FungibleToken,
    steps_since_tge: U64,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(postfix: Option<String>) -> Self {
        Self {
            oracles: UnorderedSet::new(b"s"),
            token: FungibleToken::new(b"t", postfix),
            steps_since_tge: U64::from(0),
        }
    }
    pub fn add_oracle(&mut self, account_id: &AccountId) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Unauthorized access! Only token owner can add oracles!"
        );
        require!(self.oracles.insert(account_id) == true, "Already exists!");
        env::log_str(&format!("Oracle {} was added", account_id));
    }

    pub fn remove_oracle(&mut self, account_id: &AccountId) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Unauthorized access! Only token owner can remove oracles!"
        );
        require!(
            self.oracles.remove(account_id) == true,
            "No such oracle was found!"
        );
        env::log_str(&format!("Oracle {} was removed", account_id));
    }

    pub fn get_oracles(&self) -> Vec<AccountId> {
        self.oracles.to_vec()
    }

    pub fn tge_mint(&mut self, account_id: &AccountId, amount: U128) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Unauthorized access! Only token owner can do TGE!"
        );
        internal_deposit(&mut self.token, &account_id, amount.0);
        FtMint {
            owner_id: account_id,
            amount: &amount,
            memo: None,
        }
        .emit()
    }

    pub fn tge_mint_batch(&mut self, batch: Vec<(AccountId, U128)>) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Unauthorized access! Only token owner can do TGE!"
        );
        let mut events = Vec::with_capacity(batch.len());
        for i in 0..batch.len() {
            internal_deposit(&mut self.token, &batch[i].0, batch[i].1 .0);
            events.push(FtMint {
                owner_id: &batch[i].0,
                amount: &batch[i].1,
                memo: None,
            })
        }
        if !events.is_empty() {
            FtMint::emit_many(events.as_slice());
        }
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

    pub fn get_steps_since_tge(&self) -> U64 {
        self.steps_since_tge
    }

    pub fn record_batch(&mut self, steps_batch: Vec<(AccountId, u16)>) {
        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access! Only oracle can call that!"
        );
        let mut oracle_fee: U128 = U128(0);
        let mut sweats: Vec<U128> = Vec::with_capacity(steps_batch.len() + 1);
        let mut events = Vec::with_capacity(steps_batch.len() + 1);
        for i in 0..steps_batch.len() {
            let sweat_to_mint: u128 = self.formula(self.steps_since_tge, steps_batch[i].1).0;
            let trx_oracle_fee: u128 = sweat_to_mint * 5 / 100;
            let minted_to_user: u128 = sweat_to_mint - trx_oracle_fee;
            oracle_fee.0 = oracle_fee.0 + trx_oracle_fee;
            internal_deposit(&mut self.token, &steps_batch[i].0, minted_to_user);
            sweats.push(U128(minted_to_user));
            self.steps_since_tge.0 += steps_batch[i].1 as u64;
        }
        for i in 0..steps_batch.len() {
            events.push(FtMint {
                owner_id: &steps_batch[i].0,
                amount: &sweats[i],
                memo: None,
            });
        }
        internal_deposit(
            &mut self.token,
            &env::predecessor_account_id(),
            oracle_fee.0,
        );
        let oracle_event = FtMint {
            owner_id: &env::predecessor_account_id(),
            amount: &oracle_fee,
            memo: None,
        };
        events.push(oracle_event);
        FtMint::emit_many(events.as_slice());
    }

    pub fn formula(&self, steps_since_tge: U64, steps: u16) -> U128 {
        U128(math::formula(steps_since_tge.0 as f64, steps as f64))
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

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId};
    const EPS: f64 = 0.00001;

    fn sweat_the_token() -> AccountId {
        AccountId::new_unchecked("sweat_the_token".to_string())
    }
    fn sweat_oracle() -> AccountId {
        AccountId::new_unchecked("sweat_the_oracle".to_string())
    }
    fn user1() -> AccountId {
        AccountId::new_unchecked("sweat_user1".to_string())
    }
    fn user2() -> AccountId {
        AccountId::new_unchecked("sweat_user2".to_string())
    }

    fn get_context(owner: AccountId, sender: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(owner.clone())
            .signer_account_id(sender.clone())
            .predecessor_account_id(sender)
            .attached_deposit(1);
        builder
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only token owner can add oracles!"#)]
    fn add_oracle_access() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        testing_env!(get_context(sweat_the_token(), sweat_oracle()).build());
        token.add_oracle(&sweat_oracle());
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only token owner can remove oracles!"#)]
    fn remove_oracle_access() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        testing_env!(get_context(sweat_the_token(), sweat_oracle()).build());
        token.remove_oracle(&sweat_oracle());
    }

    #[test]
    #[should_panic(expected = r#"Already exists!"#)]
    fn add_same_oracle() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.add_oracle(&sweat_oracle());
    }

    #[test]
    #[should_panic(expected = r#"No such oracle was found!"#)]
    fn remove_fake_oracle() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        token.remove_oracle(&user1());
    }

    #[test]
    fn add_remove_oracle() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        token.remove_oracle(&sweat_oracle());
        assert_eq!(true, token.get_oracles().is_empty());
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only oracle can call that!"#)]
    fn mint_steps_access_1() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        token.record_batch(vec![(user1(), 10_000), (user2(), 10_000)]);
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only oracle can call that!"#)]
    fn minting_steps_access_2() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        testing_env!(get_context(sweat_the_token(), user1()).build());
        token.record_batch(vec![(user1(), 10_000), (user2(), 10_000)]);
    }

    #[test]
    fn oracle_fee_test() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(U64(0), token.get_steps_since_tge());
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        testing_env!(get_context(sweat_the_token(), sweat_oracle()).build());
        token.record_batch(vec![(user1(), 10_000), (user2(), 10_000)]);
        assert_eq!(
            true,
            (9.499999991723028480 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs()
                < EPS
        );
        assert_eq!(
            true,
            (9.499999975169081549 - token.token.ft_balance_of(user2()).0 as f64 / 1e+18).abs()
                < EPS
        );
        assert_eq!(
            true,
            (0.999999998257479475 - token.token.ft_balance_of(sweat_oracle()).0 as f64 / 1e+18)
                .abs()
                < EPS
        );
        assert_eq!(U64(2 * 10_000), token.get_steps_since_tge());
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only token owner can do TGE!"#)]
    fn tge_access_1() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        testing_env!(get_context(sweat_the_token(), sweat_oracle()).build());
        token.tge_mint(&user1(), U128(9499999991723028480));
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only token owner can do TGE!"#)]
    fn tge_access_2() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        token.add_oracle(&sweat_oracle());
        testing_env!(get_context(sweat_the_token(), user1()).build());
        token.tge_mint_batch(vec![
            (user1(), U128(9499999991723028480)),
            (user2(), U128(9499999991723028480)),
        ]);
    }

    #[test]
    fn tge_liquid() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint(&user1(), U128(9499999991723028480));
        assert_eq!(
            true,
            (9.499999991723028480 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs()
                < EPS
        );
    }

    #[test]
    fn tge_liquid_batch() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint_batch(vec![
            (user1(), U128(9499999991723028480)),
            (user2(), U128(9499999991723028480)),
        ]);
        assert_eq!(
            true,
            (9.499999991723028480 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs()
                < EPS
        );
        assert_eq!(
            true,
            (9.499999975169081549 - token.token.ft_balance_of(user2()).0 as f64 / 1e+18).abs()
                < EPS
        );
    }

    #[test]
    fn burn() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint(&user1(), U128(9499999991723028480));
        testing_env!(get_context(sweat_the_token(), user1()).build());
        token.burn(&U128(9499999991723028480));
        assert_eq!(
            true,
            (0.0 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS
        );
    }

    #[test]
    #[should_panic(expected = r#"The account sweat_user2 is not registered"#)]
    fn transfer_to_unregistered() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint(&user1(), U128(9499999991723028480));
        testing_env!(get_context(sweat_the_token(), user1()).build());

        token
            .token
            .ft_transfer(user2(), U128(9499999991723028480), None);

        assert_eq!(
            true,
            (0.0 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS
        );

        assert_eq!(
            true,
            (9.499999991723028480 - token.token.ft_balance_of(user2()).0 as f64 / 1e+18).abs()
                < EPS
        );
    }

    #[test]
    fn transfer_to_registered() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert_eq!(true, token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint_batch(vec![
            (user1(), U128(9499999991723028480)),
            (user2(), U128(9499999991723028480)),
        ]);
        testing_env!(get_context(sweat_the_token(), user1()).build());

        token
            .token
            .ft_transfer(user2(), U128(9499999991723028480), None);

        assert_eq!(
            true,
            (0.0 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS
        );

        assert_eq!(
            true,
            (9.499999991723028480 * 2.0 - token.token.ft_balance_of(user2()).0 as f64 / 1e+18)
                .abs()
                < EPS
        );
    }
}
