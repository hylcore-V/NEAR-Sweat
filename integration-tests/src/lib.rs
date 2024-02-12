#![cfg(test)]

use integration_utils::misc::ToNear;
use sweat_model::{FungibleTokenCoreIntegration, SweatApiIntegration, SweatDeferIntegration};

use crate::prepare::{prepare_contract, IntegrationContext};

mod callback_attack;
mod common;
mod defer;
mod formula;
mod interface;
mod measure;
mod mint;
mod prepare;
mod transfer;

#[tokio::test]
async fn happy_flow() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let alice = context.alice().await?;
    let oracle = context.oracle().await?;

    assert_eq!(
        99999995378125008,
        context.ft_contract().formula(100_000.into(), 100).await?.0
    );

    context
        .ft_contract()
        .tge_mint(&alice.to_near(), 100_000_000.into())
        .await?;

    assert_eq!(
        100_000_000,
        context.ft_contract().ft_balance_of(alice.to_near()).await?.0
    );

    context
        .ft_contract()
        .defer_batch(
            vec![(alice.to_near(), 1000)],
            context.claim_contract().as_account().to_near(),
        )
        .with_user(&oracle)
        .await?;

    Ok(())
}
