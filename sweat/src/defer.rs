use near_contract_standards::fungible_token::events::FtMint;
use near_sdk::{
    env, env::panic_str, ext_contract, is_promise_success, json_types::U128, near_bindgen, require, serde_json::json,
    AccountId, Gas, Promise, PromiseOrValue,
};
use sweat_model::SweatDefer;

use crate::{internal_deposit, Contract, ContractExt};

const ONE_GGAS: u64 = Gas::ONE_TERA.0 / 1000;

#[near_bindgen]
impl SweatDefer for Contract {
    fn defer_batch(&mut self, steps_batch: Vec<(AccountId, u32)>, holding_account_id: AccountId) -> PromiseOrValue<()> {
        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access! Only oracle can call that!"
        );

        let batch_len = steps_batch.len() as u64;

        let mut accounts_tokens: Vec<(AccountId, U128)> = Vec::new();
        let mut total_effective: U128 = U128(0);
        let mut total_fee: U128 = U128(0);

        for (account_id, step_count) in steps_batch {
            let (amount, fee) = self.calculate_tokens_amount(step_count);
            self.steps_since_tge.0 += u64::from(step_count);

            accounts_tokens.push((account_id, U128(amount)));
            total_effective.0 += amount;
            total_fee.0 += fee;
        }

        let hold_arguments = json!({
            "amounts": accounts_tokens,
        });

        // These values calculated in `measure_record_batch_for_hold_test` in claim contract.
        let record_batch_for_hold_gas = Gas::ONE_TERA * 8 + Gas(batch_len * ONE_GGAS * 320);

        Promise::new(holding_account_id.clone())
            .function_call(
                "record_batch_for_hold".to_string(),
                hold_arguments.to_string().into_bytes(),
                0,
                record_batch_for_hold_gas,
            )
            .then(
                ext_ft_transfer_callback::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 5)
                    .on_record(
                        holding_account_id,
                        total_effective,
                        env::predecessor_account_id(),
                        total_fee,
                    ),
            )
            .into()
    }
}

#[ext_contract(ext_ft_transfer_callback)]
pub trait FungibleTokenTransferCallback {
    fn on_record(&mut self, receiver_id: AccountId, amount: U128, fee_account_id: AccountId, fee: U128);
}

#[near_bindgen]
impl FungibleTokenTransferCallback for Contract {
    fn on_record(&mut self, receiver_id: AccountId, amount: U128, fee_account_id: AccountId, fee: U128) {
        if !self.oracles.contains(&env::signer_account_id()) {
            panic_str("The operation can be only initiated by an oracle");
        }

        if !is_promise_success() {
            panic_str("Failed to record data in holding account");
        }

        let mut events: Vec<FtMint> = Vec::with_capacity(2);

        internal_deposit(&mut self.token, &fee_account_id, fee.0);
        events.push(FtMint {
            owner_id: &fee_account_id,
            amount: &fee,
            memo: None,
        });

        internal_deposit(&mut self.token, &receiver_id, amount.0);
        events.push(FtMint {
            owner_id: &receiver_id,
            amount: &amount,
            memo: None,
        });

        FtMint::emit_many(&events);
    }
}
