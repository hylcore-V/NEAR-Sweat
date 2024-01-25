#[macro_use]
extern crate static_assertions;

use near_contract_standards::fungible_token::{
    events::{FtBurn, FtMint},
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
    FungibleToken,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedSet,
    env,
    json_types::{U128, U64},
    near_bindgen, require, AccountId, Balance, PanicOnDefault, PromiseOrValue,
};
use sweat_model::{Payout, SweatApi};

mod defer;
mod integration;
mod math;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    oracles: UnorderedSet<AccountId>,
    token: FungibleToken,
    steps_since_tge: U64,
}

#[near_bindgen]
impl SweatApi for Contract {
    #[init]
    fn new(postfix: Option<String>) -> Self {
        Self {
            oracles: UnorderedSet::new(b"s"),
            token: FungibleToken::new(b"t", postfix),
            steps_since_tge: U64::from(0),
        }
    }
    fn add_oracle(&mut self, account_id: &AccountId) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Unauthorized access! Only token owner can add oracles!"
        );
        require!(self.oracles.insert(account_id), "Already exists!");
        env::log_str(&format!("Oracle {account_id} was added"));
    }

    fn remove_oracle(&mut self, account_id: &AccountId) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Unauthorized access! Only token owner can remove oracles!"
        );
        require!(self.oracles.remove(account_id), "No such oracle was found!");
        env::log_str(&format!("Oracle {account_id} was removed"));
    }

    fn get_oracles(&self) -> Vec<AccountId> {
        self.oracles.to_vec()
    }

    fn tge_mint(&mut self, account_id: &AccountId, amount: U128) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Unauthorized access! Only token owner can do TGE!"
        );
        internal_deposit(&mut self.token, account_id, amount.0);
        FtMint {
            owner_id: account_id,
            amount: &amount,
            memo: None,
        }
        .emit();
    }

    fn tge_mint_batch(&mut self, batch: Vec<(AccountId, U128)>) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Unauthorized access! Only token owner can do TGE!"
        );
        let mut events = Vec::with_capacity(batch.len());
        for (account_id, steps_count) in &batch {
            // let steps_count = steps_count.0;
            internal_deposit(&mut self.token, account_id, steps_count.0);

            let event = FtMint {
                owner_id: account_id,
                amount: steps_count,
                memo: None,
            };
            events.push(event);
        }
        if !events.is_empty() {
            FtMint::emit_many(events.as_slice());
        }
    }

    fn burn(&mut self, amount: &U128) {
        self.token.internal_withdraw(&env::predecessor_account_id(), amount.0);
        FtBurn {
            amount,
            owner_id: &env::predecessor_account_id(),
            memo: None,
        }
        .emit();
    }

    fn get_steps_since_tge(&self) -> U64 {
        self.steps_since_tge
    }

    fn record_batch(&mut self, steps_batch: Vec<(AccountId, u32)>) {
        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access! Only oracle can call that!"
        );
        let mut oracle_fee: U128 = U128(0);
        let mut sweats: Vec<U128> = Vec::with_capacity(steps_batch.len() + 1);
        let mut events = Vec::with_capacity(steps_batch.len() + 1);

        for (account_id, steps_count) in &steps_batch {
            let (minted_to_user, trx_oracle_fee) = self.calculate_tokens_amount(*steps_count);
            oracle_fee.0 += trx_oracle_fee;
            internal_deposit(&mut self.token, account_id, minted_to_user);

            sweats.push(U128(minted_to_user));
            self.steps_since_tge.0 += u64::from(*steps_count);
        }
        for i in 0..steps_batch.len() {
            events.push(FtMint {
                owner_id: &steps_batch[i].0,
                amount: &sweats[i],
                memo: None,
            });
        }

        internal_deposit(&mut self.token, &env::predecessor_account_id(), oracle_fee.0);
        let oracle_event = FtMint {
            owner_id: &env::predecessor_account_id(),
            amount: &oracle_fee,
            memo: None,
        };
        events.push(oracle_event);
        FtMint::emit_many(events.as_slice());
    }

    #[allow(clippy::cast_precision_loss)]
    fn formula(&self, steps_since_tge: U64, steps: u32) -> U128 {
        U128(math::formula(steps_since_tge.0 as f64, f64::from(steps)))
    }
}

impl Contract {
    pub(crate) fn calculate_tokens_amount(&self, steps: u32) -> (u128, u128) {
        let sweat_to_mint: u128 = self.formula(self.steps_since_tge, steps).0;
        let payout = Payout::from(sweat_to_mint);

        (payout.amount_for_user, payout.fee)
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
    use near_contract_standards::fungible_token::core::FungibleTokenCore;
    use near_sdk::{
        json_types::{U128, U64},
        test_utils::VMContextBuilder,
        testing_env, AccountId,
    };
    use sweat_model::SweatApi;

    use crate::Contract;

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
        assert!(token.get_oracles().is_empty());
        testing_env!(get_context(sweat_the_token(), sweat_oracle()).build());
        token.add_oracle(&sweat_oracle());
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only token owner can remove oracles!"#)]
    fn remove_oracle_access() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
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
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.add_oracle(&sweat_oracle());
    }

    #[test]
    #[should_panic(expected = r#"No such oracle was found!"#)]
    fn remove_fake_oracle() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        token.remove_oracle(&user1());
    }

    #[test]
    fn add_remove_oracle() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        token.remove_oracle(&sweat_oracle());
        assert!(token.get_oracles().is_empty());
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only oracle can call that!"#)]
    fn mint_steps_access_1() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        token.record_batch(vec![(user1(), 10_000), (user2(), 10_000)]);
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only oracle can call that!"#)]
    fn minting_steps_access_2() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
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
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        assert_eq!(vec![sweat_oracle()], token.get_oracles());
        testing_env!(get_context(sweat_the_token(), sweat_oracle()).build());
        token.record_batch(vec![(user1(), 10_000), (user2(), 10_000)]);
        assert!((9.499_999_991_723_028 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS);
        assert!((9.499_999_975_169_082 - token.token.ft_balance_of(user2()).0 as f64 / 1e+18).abs() < EPS);
        assert!((0.999_999_998_257_479_4 - token.token.ft_balance_of(sweat_oracle()).0 as f64 / 1e+18).abs() < EPS);
        assert_eq!(U64(2 * 10_000), token.get_steps_since_tge());
    }

    #[test]
    #[should_panic(expected = r#"Unauthorized access! Only token owner can do TGE!"#)]
    fn tge_access_1() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
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
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint(&user1(), U128(9499999991723028480));
        assert!((9.499_999_991_723_028 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS);
    }

    #[test]
    fn tge_liquid_batch() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint_batch(vec![
            (user1(), U128(9499999991723028480)),
            (user2(), U128(9499999991723028480)),
        ]);
        assert!((9.499_999_991_723_028 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS);
        assert!((9.499_999_975_169_082 - token.token.ft_balance_of(user2()).0 as f64 / 1e+18).abs() < EPS);
    }

    #[test]
    fn burn() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint(&user1(), U128(9499999991723028480));
        testing_env!(get_context(sweat_the_token(), user1()).build());
        token.burn(&U128(9499999991723028480));
        assert!((0.0 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS);
    }

    #[test]
    #[should_panic(expected = r#"The account sweat_user2 is not registered"#)]
    fn transfer_to_unregistered() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint(&user1(), U128(9499999991723028480));
        testing_env!(get_context(sweat_the_token(), user1()).build());

        token.token.ft_transfer(user2(), U128(9499999991723028480), None);

        assert!((0.0 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS);

        assert!((9.499_999_991_723_028 - token.token.ft_balance_of(user2()).0 as f64 / 1e+18).abs() < EPS);
    }

    #[test]
    fn transfer_to_registered() {
        testing_env!(get_context(sweat_the_token(), sweat_the_token()).build());
        let mut token = Contract::new(Some(".u.sweat".to_string()));
        assert!(token.get_oracles().is_empty());
        token.add_oracle(&sweat_oracle());
        token.tge_mint_batch(vec![
            (user1(), U128(9499999991723028480)),
            (user2(), U128(9499999991723028480)),
        ]);
        testing_env!(get_context(sweat_the_token(), user1()).build());

        token.token.ft_transfer(user2(), U128(9499999991723028480), None);

        assert!((0.0 - token.token.ft_balance_of(user1()).0 as f64 / 1e+18).abs() < EPS);

        assert!((9.499_999_991_723_028 * 2.0 - token.token.ft_balance_of(user2()).0 as f64 / 1e+18).abs() < EPS);
    }
}
