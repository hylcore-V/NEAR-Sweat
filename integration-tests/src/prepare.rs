use async_trait::async_trait;
use integration_utils::{integration_contract::IntegrationContract, misc::ToNear};
use near_workspaces::{Account, Contract};
use sweat_integration::{SweatFt, FT_CONTRACT};
use sweat_model::{StorageManagementIntegration, SweatApiIntegration};

const HOLDING_STUB_CONTRACT: &str = "defer_stub";

pub type Context = integration_utils::context::Context<near_workspaces::network::Sandbox>;

#[async_trait]
pub trait IntegrationContext {
    async fn oracle(&mut self) -> anyhow::Result<Account>;
    async fn alice(&mut self) -> anyhow::Result<Account>;
    async fn bob(&mut self) -> anyhow::Result<Account>;
    fn ft_contract(&self) -> SweatFt;

    fn holding_contract(&self) -> &Contract;
}

#[async_trait]
impl IntegrationContext for Context {
    async fn oracle(&mut self) -> anyhow::Result<Account> {
        self.account("oracle").await
    }

    async fn alice(&mut self) -> anyhow::Result<Account> {
        self.account("alice").await
    }

    async fn bob(&mut self) -> anyhow::Result<Account> {
        self.account("bob").await
    }

    fn ft_contract(&self) -> SweatFt {
        SweatFt::with_contract(&self.contracts[FT_CONTRACT])
    }

    fn holding_contract(&self) -> &Contract {
        &self.contracts[HOLDING_STUB_CONTRACT]
    }
}

pub async fn prepare_contract() -> anyhow::Result<Context> {
    let mut context = Context::new(&[FT_CONTRACT, HOLDING_STUB_CONTRACT], "build".into()).await?;
    let oracle = context.oracle().await?;
    let alice = context.alice().await?;

    context.ft_contract().new(".u.sweat.testnet".to_string().into()).await?;

    context
        .ft_contract()
        .storage_deposit(oracle.to_near().into(), None)
        .await?;

    context
        .ft_contract()
        .storage_deposit(alice.to_near().into(), None)
        .await?;

    context.ft_contract().add_oracle(&oracle.to_near()).await?;

    let holding_contract_init_result = context
        .holding_contract()
        .call("new")
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    println!("Initialized holding contract: {:?}", holding_contract_init_result);

    context
        .ft_contract()
        .storage_deposit(context.holding_contract().as_account().to_near().into(), None)
        .await?;

    Ok(context)
}
