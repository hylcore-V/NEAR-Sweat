use near_contract_standards::fungible_token::events::FtMint;
use near_sdk::{
    env, env::panic_str, ext_contract, is_promise_success, json_types::U128, near_bindgen, require, serde_json::json,
    AccountId, Gas, Promise, PromiseOrValue,
};
use sweat_model::SweatDefer;

use crate::{internal_deposit, Contract, ContractExt};

const GAS_FOR_DEFER_CALLBACK: Gas = Gas(5 * Gas::ONE_TERA.0);
const GAS_FOR_DEFER: Gas = Gas(30 * Gas::ONE_TERA.0);

#[near_bindgen]
impl SweatDefer for Contract {
    fn defer_batch(&mut self, steps_batch: Vec<(AccountId, u32)>, holding_account_id: AccountId) -> PromiseOrValue<()> {
        require!(
            env::prepaid_gas() > GAS_FOR_DEFER,
            "Not enough gas to complete the operation"
        );

        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access! Only oracle can call that!"
        );

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

        let record_batch_for_hold_gas = Gas(env::prepaid_gas()
            .0
            .checked_sub(GAS_FOR_DEFER.0)
            .unwrap_or_else(|| panic_str("Prepaid gas overflow")));

        Promise::new(holding_account_id.clone())
            .function_call(
                "record_batch_for_hold".to_string(),
                hold_arguments.to_string().into_bytes(),
                0,
                record_batch_for_hold_gas,
            )
            .then(
                ext_ft_transfer_callback::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_DEFER_CALLBACK)
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
    #[private]
    fn on_record(&mut self, receiver_id: AccountId, amount: U128, fee_account_id: AccountId, fee: U128) {
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
