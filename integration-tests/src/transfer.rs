use integration_utils::misc::ToNear;
use near_sdk::json_types::U128;
use sweat_model::{FungibleTokenCoreIntegration, StorageManagementIntegration, SweatApiIntegration};

use crate::prepare::{prepare_contract, IntegrationContext};

#[tokio::test]
async fn test_transfer() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;
    let oracle = context.oracle().await?;
    let alice = context.alice().await?;
    let bob = context.bob().await?;

    context
        .ft_contract()
        .record_batch(vec![(alice.to_near(), 10_000)])
        .with_user(&oracle)
        .await?;

    // This will fail because storage is not registered for this new account
    let res = context
        .ft_contract()
        .ft_transfer(bob.to_near(), U128(9499999991723028480), None)
        .with_user(&alice)
        .await;
    assert!(res.is_err());

    let res = context.ft_contract().storage_deposit(Some(bob.to_near()), None).await;
    assert!(res.is_ok());

    let alice_balance = context.ft_contract().ft_balance_of(alice.to_near()).await?;
    assert_ne!(U128(0), alice_balance);

    // Transfer all tokens from alice to new account
    context
        .ft_contract()
        .ft_transfer(bob.to_near(), alice_balance, None)
        .with_user(&alice)
        .await?;

    let alice_balance_updated = context.ft_contract().ft_balance_of(alice.to_near()).await?;
    assert_eq!(U128(0), alice_balance_updated);

    let bob_balance = context.ft_contract().ft_balance_of(bob.to_near()).await?;
    assert_eq!(alice_balance, bob_balance);

    Ok(())
}
