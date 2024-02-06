#![allow(clippy::new_ret_no_self, clippy::wrong_self_convention)]

use integration_trait::make_integration_version;
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds};
use near_sdk::{
    json_types::{U128, U64},
    AccountId, PromiseOrValue,
};

#[cfg(feature = "integration-test")]
pub struct SweatContract<'a> {
    pub contract: &'a near_workspaces::Contract,
}

#[make_integration_version]
pub trait SweatApi {
    fn new(postfix: Option<String>) -> Self;
    fn add_oracle(&mut self, account_id: &AccountId);
    fn remove_oracle(&mut self, account_id: &AccountId);
    fn get_oracles(&self) -> Vec<AccountId>;
    fn tge_mint(&mut self, account_id: &AccountId, amount: U128);
    fn tge_mint_batch(&mut self, batch: Vec<(AccountId, U128)>);
    fn burn(&mut self, amount: &U128);
    fn get_steps_since_tge(&self) -> U64;
    fn record_batch(&mut self, steps_batch: Vec<(AccountId, u32)>);
    fn formula(&self, steps_since_tge: U64, steps: u32) -> U128;
}

#[make_integration_version]
pub trait SweatDefer {
    fn defer_batch(&mut self, steps_batch: Vec<(AccountId, u32)>, holding_account_id: AccountId) -> PromiseOrValue<()>;
}

/// Copy of near_sdk trait to use in integration tests
#[make_integration_version]
pub trait FungibleTokenCore {
    #[deposit_one_yocto]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;
    fn ft_total_supply(&self) -> U128;
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

/// Copy of near_sdk trait to use in integration tests
#[make_integration_version]
pub trait StorageManagement {
    // if `registration_only=true` MUST refund above the minimum balance if the account didn't exist and
    //     refund full deposit if the account exists.
    #[deposit_yocto = near_sdk::env::storage_byte_cost() * 125]
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance;

    /// Withdraw specified amount of available Ⓝ for predecessor account.
    ///
    /// This method is safe to call. It MUST NOT remove data.
    ///
    /// `amount` is sent as a string representing an unsigned 128-bit integer. If
    /// omitted, contract MUST refund full `available` balance. If `amount` exceeds
    /// predecessor account's available balance, contract MUST panic.
    ///
    /// If predecessor account not registered, contract MUST panic.
    ///
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted
    /// function-call access-key call (UX wallet security)
    ///
    /// Returns the StorageBalance structure showing updated balances.
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance;

    /// Unregisters the predecessor account and returns the storage NEAR deposit back.
    ///
    /// If the predecessor account is not registered, the function MUST return `false` without panic.
    ///
    /// If `force=true` the function SHOULD ignore account balances (burn them) and close the account.
    /// Otherwise, MUST panic if caller has a positive registered balance (eg token holdings) or
    ///     the contract doesn't support force unregistration.
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted function-call access-key call
    /// (UX wallet security)
    /// Returns `true` iff the account was unregistered.
    /// Returns `false` iff account was not registered before.
    fn storage_unregister(&mut self, force: Option<bool>) -> bool;

    fn storage_balance_bounds(&self) -> StorageBalanceBounds;

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance>;
}

#[make_integration_version]
pub trait IntegrationTestMethods {
    fn calculate_payout_with_fee_for_batch(&self, batch_size: u32, claim_amount: u32) -> (U128, U128);
}

pub struct Payout {
    pub amount_for_user: u128,
    pub fee: u128,
}

impl From<u128> for Payout {
    fn from(value: u128) -> Self {
        let fee = (value * 5).div_ceil(100);

        Self {
            fee,
            amount_for_user: value - fee,
        }
    }
}
