use anyhow::Result;
use async_trait::async_trait;
use integration_utils::misc::ToNear;
use near_sdk::serde_json::json;
use near_workspaces::{Account, Contract};
use sweat_model::{StorageManagementIntegration, SweatApiIntegration, SweatContract};

const CLAIM_CONTRACT: &str = "sweat_claim";
const HOLDING_STUB_CONTRACT: &str = "exploit_stub";
const FT_CONTRACT: &str = "sweat";

pub type Context = integration_utils::context::Context<near_workspaces::network::Sandbox>;

#[async_trait]
pub trait IntegrationContext {
    async fn oracle(&mut self) -> Result<Account>;
    async fn alice(&mut self) -> Result<Account>;
    async fn bob(&mut self) -> Result<Account>;
    async fn long_account_name(&mut self) -> Result<Account>;

    fn ft_contract(&self) -> SweatContract;
    fn claim_contract(&self) -> &Contract;
    fn stub_contract(&self) -> &Contract;
}

#[async_trait]
impl IntegrationContext for Context {
    async fn oracle(&mut self) -> Result<Account> {
        self.account("oracle").await
    }

    async fn alice(&mut self) -> Result<Account> {
        self.account("alice").await
    }

    async fn bob(&mut self) -> Result<Account> {
        self.account("bob").await
    }

    async fn long_account_name(&mut self) -> Result<Account> {
        self.account("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").await
    }

    fn ft_contract(&self) -> SweatContract {
        SweatContract {
            contract: &self.contracts[FT_CONTRACT],
        }
    }

    fn claim_contract(&self) -> &Contract {
        &self.contracts[CLAIM_CONTRACT]
    }

    fn stub_contract(&self) -> &Contract {
        &self.contracts[HOLDING_STUB_CONTRACT]
    }
}

pub async fn prepare_contract() -> Result<Context> {
    let mut context = Context::new(
        &[FT_CONTRACT, CLAIM_CONTRACT, HOLDING_STUB_CONTRACT],
        true,
        "build-integration".into(),
    )
    .await?;
    let oracle = context.oracle().await?;
    let alice = context.alice().await?;
    let long = context.long_account_name().await?;
    let token_account_id = context.ft_contract().contract.as_account().to_near();

    context.ft_contract().new(".u.sweat.testnet".to_string().into()).await?;

    context
        .ft_contract()
        .storage_deposit(oracle.to_near().into(), None)
        .await?;

    context
        .ft_contract()
        .storage_deposit(alice.to_near().into(), None)
        .await?;

    context
        .ft_contract()
        .storage_deposit(long.to_near().into(), None)
        .await?;

    context.ft_contract().add_oracle(&oracle.to_near()).await?;

    let claim_contract_result = context
        .claim_contract()
        .call("init")
        .args_json(json!({ "token_account_id": token_account_id }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    println!("Initialized claim contract: {:?}", claim_contract_result);

    let exploit_stup_contract_result = context
        .stub_contract()
        .call("new")
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    println!("Initialized exploit stub contract: {:?}", exploit_stup_contract_result);

    context
        .claim_contract()
        .call("add_oracle")
        .args_json(json!({ "account_id": token_account_id }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    context
        .ft_contract()
        .storage_deposit(context.claim_contract().as_account().to_near().into(), None)
        .await?;

    Ok(context)
}
