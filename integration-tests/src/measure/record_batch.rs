#![cfg(test)]

use std::future::IntoFuture;

use anyhow::Result;
use integration_utils::measure::outcome_storage::OutcomeStorage;
use near_workspaces::types::Gas;
use sweat_model::SweatApiIntegration;

use crate::{prepare::IntegrationContext, prepare_contract};

#[ignore]
#[tokio::test]
async fn single_record_batch() -> anyhow::Result<()> {
    let gas = measure_record_batch().await?;

    dbg!(&gas);

    Ok(())
}

async fn measure_record_batch() -> Result<Gas> {
    let mut context = prepare_contract().await?;

    let oracle = context.oracle().await?;

    let (gas, _) = OutcomeStorage::measure_total(
        &oracle,
        context
            .ft_contract()
            .record_batch(Default::default())
            .with_user(&oracle)
            .into_future(),
    )
    .await?;

    Ok(gas)
}
