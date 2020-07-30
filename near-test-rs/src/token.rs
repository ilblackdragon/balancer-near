use near_sdk::AccountId;
use near_sdk::json_types::U128;
use serde_json::json;

use crate::test_user::{TestRuntime, to_yocto};

/// Interface for fungible token contract to test in standalone mode.
pub struct TokenContract {
    pub contract_id: AccountId,
}

impl TokenContract {
    pub fn new(
        runtime: &mut TestRuntime,
        signer_id: &AccountId,
        wasm_bytes: &[u8],
        contract_id: AccountId,
        owner_id: &AccountId,
        total_supply: &str,
    ) -> Self {
        let _ = runtime.deploy(signer_id.clone(), contract_id.clone(), wasm_bytes, json!({"owner_id": owner_id.clone(), "total_supply": U128::from(to_yocto(total_supply))})).unwrap();
        Self { contract_id }
    }

    pub fn mint(&self, runtime: &mut TestRuntime, signer_id: &AccountId, account_id: &AccountId, amount: &str) {
        let _ = runtime
            .call(
                signer_id.clone(),
                self.contract_id.clone(),
                "mint",
                json!({"account_id": account_id, "amount": U128::from(to_yocto(amount))}),
                0,
            )
            .unwrap();
    }
}
