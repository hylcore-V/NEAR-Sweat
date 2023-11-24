#![cfg(test)]

use integration_utils::{integration_contract::IntegrationContract, misc::ToNear};
use near_sdk::json_types::U64;
use sweat_model::{FungibleTokenCoreIntegration, SweatApiIntegration, SweatDeferIntegration};

use crate::prepare::{prepare_contract, IntegrationContext};

#[tokio::test]
async fn test_defer() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;
    let oracle = context.oracle().await?;
    let alice = context.alice().await?;
    let holding_account = context.holding_contract().as_account().to_near();

    let target_amount = context.ft_contract().formula(U64(0), 10_000).await?;
    assert_ne!(0, target_amount.0);

    context
        .ft_contract()
        .with_user(&oracle)
        .defer_batch(vec![(alice.to_near(), 10_000)], holding_account.clone())
        .await?;

    let alice_balance = context.ft_contract().ft_balance_of(alice.to_near()).await?;
    assert_eq!(0, alice_balance.0);

    let holder_balance = context.ft_contract().ft_balance_of(holding_account.clone()).await?;
    assert_eq!(target_amount.0 * 95 / 100, holder_balance.0);

    let oracle_balance = context.ft_contract().ft_balance_of(oracle.to_near()).await?;
    assert_eq!(target_amount.0 * 5 / 100, oracle_balance.0);

    Ok(())
}
