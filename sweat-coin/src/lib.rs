use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault, PromiseOrValue, env, Balance, BlockHeight};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    oracle_id: AccountId,
    limit_per_day: Balance,
    token: FungibleToken,
    last_block_update: BlockHeight,
    avg_minted: Balance,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(oracle_id: AccountId, limit_per_day: U128) -> Self {
        Self {
            oracle_id,
            limit_per_day: limit_per_day.into(),
            token: FungibleToken::new(b"t".to_vec()),
            last_block_update: 0,
            avg_minted: 0,
        }
    }

    fn internal_step_to_amount(&self, steps: u64) -> Balance {
        steps as u128
    }

    pub fn batch_record(&mut self, steps_batch: Vec<(ValidAccountId, u64)>) {
        assert_eq!(env::predecessor_account_id(), self.oracle_id);
        let mut amount_minted = 0;
        for (account_id, steps) in steps_batch.into_iter() {
            if !self.token.accounts.contains_key(account_id.as_ref()) {
                self.token.internal_register_account(account_id.as_ref());
            }
            let amount = self.internal_step_to_amount(steps);
            amount_minted += amount;
            self.token
                .internal_deposit(account_id.as_ref(), amount);
        }
        if self.last_block_update == env::block_index() {
            self.avg_minted += amount_minted;
        } else {
            self.avg_minted = self.avg_minted * (env::block_index() - self.last_block_update) as u128 / 86400u128 + amount_minted / 86400u128;
            assert!(self.avg_minted < self.limit_per_day);
        }
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0".to_string(),
            name: "Sweat Coin".to_string(),
            symbol: "SWEAT".to_string(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 0
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env, MockedBlockchain};

    use super::*;

    // #[test]
    // fn test_basics() {
    //     let mut context = VMContextBuilder::new();
    //     testing_env!(context.build());
    //     let mut contract = Contract::new();
    //     testing_env!(context
    //         .attached_deposit(125 * env::storage_byte_cost())
    //         .build());
    //     contract.mint(accounts(0), 1_000_000.into());
    //     assert_eq!(contract.ft_balance_of(accounts(0)), 1_000_000.into());
    //
    //     testing_env!(context
    //         .attached_deposit(125 * env::storage_byte_cost())
    //         .build());
    //     contract.storage_deposit(Some(accounts(1)), None);
    //     testing_env!(context
    //         .attached_deposit(1)
    //         .predecessor_account_id(accounts(0))
    //         .build());
    //     contract.ft_transfer(accounts(1), 1_000.into(), None);
    //     assert_eq!(contract.ft_balance_of(accounts(1)), 1_000.into());
    //
    //     contract.burn(accounts(1), 500.into());
    //     assert_eq!(contract.ft_balance_of(accounts(1)), 500.into());
    // }
}
