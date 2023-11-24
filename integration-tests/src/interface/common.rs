use integration_utils::integration_contract::IntegrationContract;
use near_sdk::AccountId;
use sweat_integration::SweatFt;

pub(crate) trait ContractAccount {
    fn account(&self) -> AccountId;
}

impl ContractAccount for SweatFt<'_> {
    fn account(&self) -> AccountId {
        AccountId::new_unchecked(self.contract().as_account().id().to_string())
    }
}
