use async_trait::async_trait;
use integration_utils::{contract_call::ContractCall, integration_contract::IntegrationContract};
use near_sdk::{
    json_types::{U128, U64},
    serde_json::json,
    AccountId,
};
use near_workspaces::{types::NearToken, Contract};
use sweat_model::{
    FungibleTokenCoreIntegration, StorageManagementIntegration, SweatApiIntegration, SweatDeferIntegration,
};

pub const FT_CONTRACT: &str = "sweat";

pub struct SweatFt<'a> {
    contract: &'a Contract,
}

impl FungibleTokenCoreIntegration for SweatFt<'_> {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) -> ContractCall<()> {
        println!("▶️ Transfer {:?} fungible tokens to {}", amount, receiver_id);

        let args = json!({
            "receiver_id": receiver_id,
            "amount": amount,
            "memo": memo,
        });

        self.make_call("ft_transfer")
            .args_json(args)
            .unwrap()
            .deposit(NearToken::from_yoctonear(1))
    }

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> ContractCall<U128> {
        println!(
            "▶️ Transfer {:?} fungible tokens to {} with message: {}",
            amount, receiver_id, msg
        );

        let args = json!({
            "receiver_id": receiver_id,
            "amount": amount,
            "memo": memo,
            "msg": msg.to_string(),
        });

        self.make_call("ft_transfer_call")
            .args_json(args)
            .unwrap()
            .deposit(NearToken::from_yoctonear(1))
    }

    fn ft_total_supply(&self) -> ContractCall<U128> {
        self.make_call("ft_total_supply")
    }

    fn ft_balance_of(&self, account_id: AccountId) -> ContractCall<U128> {
        self.make_call("ft_balance_of")
            .args_json(json!({
                "account_id": account_id,
            }))
            .unwrap()
    }
}

#[async_trait]
impl StorageManagementIntegration for SweatFt<'_> {
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> ContractCall<near_contract_standards::storage_management::StorageBalance> {
        let args = json!({ "account_id": account_id, "registration_only": registration_only });
        self.make_call("storage_deposit")
            .args_json(args)
            .unwrap()
            .deposit(NearToken::from_yoctonear(near_sdk::env::storage_byte_cost() * 125))
    }

    fn storage_withdraw(
        &mut self,
        _amount: Option<U128>,
    ) -> ContractCall<near_contract_standards::storage_management::StorageBalance> {
        todo!()
    }

    fn storage_unregister(&mut self, _force: Option<bool>) -> ContractCall<bool> {
        todo!()
    }

    fn storage_balance_bounds(
        &self,
    ) -> ContractCall<near_contract_standards::storage_management::StorageBalanceBounds> {
        todo!()
    }

    fn storage_balance_of(
        &self,
        _account_id: AccountId,
    ) -> ContractCall<Option<near_contract_standards::storage_management::StorageBalance>> {
        todo!()
    }
}

#[async_trait]
impl SweatDeferIntegration for SweatFt<'_> {
    fn defer_batch(&mut self, steps_batch: Vec<(AccountId, u32)>, holding_account_id: AccountId) -> ContractCall<()> {
        self.make_call("defer_batch")
            .args_json(json!({
                "steps_batch": steps_batch,
                "holding_account_id": holding_account_id,
            }))
            .unwrap()
    }
}

#[async_trait]
impl SweatApiIntegration for SweatFt<'_> {
    fn new(&self, postfix: Option<String>) -> ContractCall<()> {
        self.make_call("new")
            .args_json(json!({
                "postfix": postfix,
            }))
            .unwrap()
    }

    fn add_oracle(&mut self, account_id: &AccountId) -> ContractCall<()> {
        self.make_call("add_oracle")
            .args_json(json!({
                "account_id": account_id,
            }))
            .unwrap()
    }

    fn remove_oracle(&mut self, _account_id: &AccountId) -> ContractCall<()> {
        todo!()
    }

    fn get_oracles(&self) -> ContractCall<Vec<AccountId>> {
        todo!()
    }

    fn tge_mint(&mut self, account_id: &AccountId, amount: U128) -> ContractCall<()> {
        self.make_call("tge_mint")
            .args_json(json!({
                "account_id": account_id,
                "amount": amount,
            }))
            .unwrap()
    }

    fn tge_mint_batch(&mut self, _batch: Vec<(AccountId, U128)>) -> ContractCall<()> {
        todo!()
    }

    fn burn(&mut self, _amount: &U128) -> ContractCall<()> {
        todo!()
    }

    fn get_steps_since_tge(&self) -> ContractCall<U64> {
        self.make_call("get_steps_since_tge")
    }

    fn record_batch(&mut self, steps_batch: Vec<(AccountId, u32)>) -> ContractCall<()> {
        self.make_call("record_batch")
            .args_json(json!({
                "steps_batch": steps_batch,
            }))
            .unwrap()
    }

    fn formula(&self, steps_since_tge: U64, steps: u32) -> ContractCall<U128> {
        self.make_call("formula")
            .args_json(json!({
                "steps_since_tge": steps_since_tge,
                "steps": steps,
            }))
            .unwrap()
    }
}

impl SweatFt<'_> {
    pub async fn formula_detailed(&self, steps_since_tge: U64, steps: u32) -> anyhow::Result<(U128, U128, U128)> {
        let token_amount = self.formula(steps_since_tge, steps).call().await?.0;
        let fee = token_amount * 5 / 100;
        let effective_amount = token_amount - fee;

        Ok((U128(fee), U128(effective_amount), U128(token_amount)))
    }
}

impl<'a> IntegrationContract<'a> for SweatFt<'a> {
    fn with_contract(contract: &'a Contract) -> Self {
        Self { contract }
    }

    fn contract(&self) -> &'a Contract {
        self.contract
    }
}
