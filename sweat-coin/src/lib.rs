use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault, PromiseOrValue, env, Balance};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    oracle_id: AccountId,
    token: FungibleToken,
    steps_from_tge: u128,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(oracle_id: AccountId) -> Self {
        Self {
            oracle_id,
            token: FungibleToken::new(b"t".to_vec()),
            steps_from_tge: 0
        }
    }

    pub fn get_steps_from_tge(&self) -> u128 {
        return self.steps_from_tge
    }

    pub fn batch_record(&mut self, steps_batch: Vec<(ValidAccountId, u32)>) {
        assert_eq!(env::predecessor_account_id(), self.oracle_id);
        for (account_id, steps) in steps_batch.into_iter() {
            if !self.token.accounts.contains_key(account_id.as_ref()) {
                self.token.internal_register_account(account_id.as_ref());
            }
            let amount = self.formula(steps);
            self.token.internal_deposit(account_id.as_ref(), amount);
            self.steps_from_tge += steps as u128;
        }
    }

    pub fn record(&mut self, account_id: ValidAccountId, steps: u32) {
        assert_eq!(env::predecessor_account_id(), self.oracle_id);
        if !self.token.accounts.contains_key(account_id.as_ref()) {
            self.token.internal_register_account(account_id.as_ref());
        }
        let amount = self.formula(steps);
        self.token.internal_deposit(account_id.as_ref(), amount);
        //self.token.ft_transfer(account_id.as_ref(), amount, "0"); 
        // :TODO: or make near vall via rpc to transfer
        self.steps_from_tge += steps as u128;
    }

    pub fn formula(&self, steps: u32) -> Balance {
        // const K:f64 = 0.9999999999999762;
        const K:f64 = 0.9999;
        // TODO: think about types here
        (   (
                K.powi((steps as i32) + (self.steps_from_tge as i32) + 1) - 
                K.powi((self.steps_from_tge as i32) + 1)
            ) / ( K - 1.) / 1000.
        ) as u128
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0".to_string(),
            name: "SWT 0.1".to_string(),
            symbol: "SWT 0.1".to_string(),
            icon: Some("http://cdn.shopify.com/s/files/1/0560/8872/3621/products/db69f0b66e63536f1bd1f716b8988fbb.jpg".to_string()),
            reference: None,
            reference_hash: None,
            decimals: 0
        }
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};
    use std::convert::TryInto;

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "robert.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "intmainreturn0.testnet".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn test_steps_from_tge() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::new("intmainreturn0.testnet".to_string());
        assert_eq!(0, contract.get_steps_from_tge());
    
        contract.record("alice.testnet".try_into().unwrap(), 10_000);
        assert_eq!(10_000, contract.get_steps_from_tge());
        
        contract.record("alice.testnet".try_into().unwrap(), 15_000);
        assert_eq!(10_000 + 15_000, contract.get_steps_from_tge());
    }

    #[test]
    fn test_formula() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::new("intmainreturn0.testnet".to_string());
        assert_eq!(0, contract.get_steps_from_tge());
        println!("get_steps_from_tge() = {}", contract.get_steps_from_tge());

        let a1 = contract.formula(10_000);
        println!("formula({}) = {}", 10_000, a1);
        
        contract.record("alice.testnet".try_into().unwrap(), 10_000);
        assert_eq!(10_000, contract.get_steps_from_tge());
        println!("get_steps_from_tge() = {}", contract.get_steps_from_tge());
        
        let a2 = contract.formula(10_000);
        println!("formula({}) = {}", 10_000, a2);

        // 0.9999 через 10к шагов сложность вырстет в 3 раза
        assert_eq!(3, a1 / a2)
    }

}