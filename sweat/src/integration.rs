#![cfg(feature = "integration-test")]

use near_sdk::{
    json_types::{U128, U64},
    near_bindgen,
};
use sweat_model::{IntegrationTestMethods, Payout, SweatApi};

use crate::{Contract, ContractExt};

#[near_bindgen]
impl IntegrationTestMethods for Contract {
    fn calculate_payout_with_fee_for_batch(&self, batch_size: u32, claim_amount: u32) -> (U128, U128) {
        let mut total_fee = 0;
        let mut total_for_user = 0;

        for i in 0..batch_size {
            let steps_since_tge = i * claim_amount;
            let steps = claim_amount;

            let minted = self.formula(U64(steps_since_tge.into()), steps).0;

            let payout = Payout::from(minted);

            total_fee += payout.fee;
            total_for_user += payout.amount_for_user;
        }

        (U128(total_fee), U128(total_for_user))
    }
}
