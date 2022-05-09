use serde_json::json;

use near_sdk::json_types::{U128, U64};
use workspaces::prelude::*;

const SWEAT_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/sweat.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // can't use sandbox on M1 because of
    // https://github.com/near/workspaces-rs/issues/110
    // ☹️
    let worker = workspaces::testnet().await?;
    let wasm = std::fs::read(SWEAT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;
    let oracle1 = worker.dev_create_account().await?;
    let oracle2 = worker.dev_create_account().await?;

    let result = contract
        .call(&worker, "new")
        .args_json(json!({
                "oracles_vec": vec![oracle1.id(), oracle2.id(), contract.id()],
        }))?
        .transact()
        .await?;
    println!("deploy & init: {:#?}", result);

    let result = worker
        .view(contract.id(), "get_steps_from_tge", Vec::new())
        .await?
        .json::<U64>()?;
    assert_eq!(result, U64(0));

    let result = contract
        .view(
            &worker,
            "formula",
            json!({
                "steps_from_tge": U64(0),
                "steps" : 10_000u32,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json::<U128>()?;
    assert_eq!(result, U128(9999999991287398400));

    let result = contract
        .call(&worker, "record_batch")
        .args_json(json!({
            "steps_batch": vec![(oracle1.id(), 10_000u32)],
        }))?
        .transact()
        .await?;
    println!("{:#?}", result);

    let result = contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": oracle1.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json::<U128>()?;
    assert_eq!(result, U128(9499999991723028480));
    assert_eq!(result, U128(9999999991287398400 * 95 / 100));

    let result = contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": contract.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json::<U128>()?;
    assert_eq!(result, U128(499999999564369920));
    assert_eq!(result, U128(9999999991287398400 * 5 / 100));

    let result = worker
        .view(contract.id(), "get_steps_from_tge", Vec::new())
        .await?
        .json::<U64>()?;
    assert_eq!(result, U64(10_000));

    Ok(())
}
