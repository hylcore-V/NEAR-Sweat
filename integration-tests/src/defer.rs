#![cfg(test)]

use integration_utils::misc::ToNear;
use sweat_model::{FungibleTokenCoreIntegration, IntegrationTestMethodsIntegration, SweatDeferIntegration};

use crate::prepare::{prepare_contract, IntegrationContext};

#[tokio::test]
async fn test_defer() -> anyhow::Result<()> {
    const BATCH_SIZE: u32 = 135;
    const CLAIM_AMOUNT: u32 = 10_000;

    let mut context = prepare_contract().await?;
    let oracle = context.oracle().await?;
    let alice = context.alice().await?;

    let claim_contract_account = context.claim_contract().as_account().to_near();

    let holder_balance = context
        .ft_contract()
        .ft_balance_of(claim_contract_account.clone())
        .await?;

    assert_eq!(holder_balance.0, 0);

    let (total_fee, total_for_user) = context
        .ft_contract()
        .calculate_payout_with_fee_for_batch(BATCH_SIZE, CLAIM_AMOUNT)
        .await?;

    let batch: Vec<_> = (0..BATCH_SIZE).map(|_| (alice.to_near(), CLAIM_AMOUNT)).collect();

    context
        .ft_contract()
        .defer_batch(batch, claim_contract_account.clone())
        .with_user(&oracle)
        .await?;

    let alice_balance = context.ft_contract().ft_balance_of(alice.to_near()).await?;
    assert_eq!(0, alice_balance.0);

    let claim_contract_balance = context
        .ft_contract()
        .ft_balance_of(claim_contract_account.clone())
        .await?;

    let oracle_balance = context.ft_contract().ft_balance_of(oracle.to_near()).await?;

    assert_eq!(oracle_balance, total_fee);
    assert_eq!(claim_contract_balance, total_for_user);

    Ok(())
}
