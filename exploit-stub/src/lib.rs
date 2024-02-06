use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    env::log_str,
    ext_contract,
    json_types::U128,
    near_bindgen, AccountId, PanicOnDefault,
};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
struct Contract {}

#[near_bindgen]
#[allow(dead_code)]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {}
    }

    #[allow(clippy::unused_self)]
    pub fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        log_str(&format!("Call record_batch_for_hold with {amounts:?}"));
    }

    #[allow(clippy::unused_self)]
    pub fn exploit_on_record(&mut self, ft_account_id: AccountId, amount: U128) {
        log_str(&format!(
            "Try to call on_record in callback, ft account = {ft_account_id}"
        ));

        let intruder_id = env::predecessor_account_id();
        ext_self::ext(env::current_account_id())
            .some_function()
            .then(ext_token::ext(ft_account_id).on_record(intruder_id.clone(), amount, intruder_id, U128(0)));
    }
}

#[ext_contract(ext_self)]
pub trait Callback {
    fn some_function(&mut self);
}

#[near_bindgen]
impl Callback for Contract {
    fn some_function(&mut self) {
        log_str("Call some_function in stub contract");
    }
}

#[ext_contract(ext_token)]
pub trait FungibleTokenTransferCallback {
    fn on_record(&mut self, receiver_id: AccountId, amount: U128, fee_account_id: AccountId, fee: U128);
}
