use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env::log_str,
    json_types::U128,
    near_bindgen, AccountId, PanicOnDefault,
};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
struct Contract {}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {}
    }

    pub fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        log_str(format!("Call record_batch_for_hold with {:?}", amounts).as_str());
    }
}
