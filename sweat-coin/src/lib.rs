use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault, PromiseOrValue, env, Balance};

near_sdk::setup_alloc!();

const ICON: &'static str = "data:image/jpeg;base64,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";

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
            icon: Some(String::from(ICON)),
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