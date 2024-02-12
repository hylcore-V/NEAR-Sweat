#![cfg(test)]

use integration_utils::misc::ToNear;
use near_sdk::{json_types::U128, serde_json::json};
use sweat_model::FungibleTokenCoreIntegration;

use crate::{
    common::PanicFinder,
    interface::common::ContractAccount,
    prepare::{prepare_contract, IntegrationContext},
};

#[tokio::test]
async fn test_call_on_record_in_callback() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let alice = context.alice().await?;

    let alice_balance_before_attack = context.ft_contract().ft_balance_of(alice.to_near()).await?;
    let ft_contract_id = context.ft_contract().account();

    let target_amount = U128(1_000_000);
    let result = alice
        .call(context.stub_contract().id(), "exploit_on_record")
        .args_json(json!({
            "ft_account_id": ft_contract_id,
            "amount": target_amount,
        }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    assert!(result.has_panic("Method on_record is private"));

    let alice_balance_after_attack = context.ft_contract().ft_balance_of(alice.to_near()).await?;
    assert_eq!(alice_balance_before_attack, alice_balance_after_attack);

    Ok(())
}

#[tokio::test]
async fn test_call_on_record_directly() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let alice = context.alice().await?;

    let intruder_id = alice.to_near();
    let result = context
        .ft_contract()
        .contract
        .as_account()
        .call(context.ft_contract().contract.id(), "on_record")
        .args_json(json!({
            "receiver_id": intruder_id,
            "amount": "1000000",
            "fee_account_id": intruder_id,
            "fee": "2000000",
        }))
        .max_gas()
        .transact()
        .await?
        .into_result();

    assert!(result.has_panic("Contract expected a result on the callback"));

    Ok(())
}
