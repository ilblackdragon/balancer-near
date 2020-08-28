use near_sdk::{AccountId, Balance};
use near_sdk::json_types::U128;
use serde_json::json;

use crate::test_user::{TestRuntime, TxResult};
use crate::units::to_yocto;

const STORAGE_PRICE_PER_BYTE: Balance = 100000000000000000000;

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

    pub fn mint(&self, runtime: &mut TestRuntime, signer_id: &AccountId, account_id: &AccountId, amount: &str) -> TxResult {
        runtime
            .call(
                signer_id.clone(),
                self.contract_id.clone(),
                "mint",
                json!({"account_id": account_id, "amount": U128::from(to_yocto(amount))}),
                0,
            )
    }

    pub fn transfer(&self, runtime: &mut TestRuntime, signer_id: &AccountId, new_owner_id: &AccountId, amount: U128) -> TxResult {
        runtime.call(signer_id.clone(), self.contract_id.clone(), "transfer", json!({"new_owner_id": new_owner_id, "amount": amount}), 0)
    }

    pub fn transfer_from(&self, runtime: &mut TestRuntime, signer_id: &AccountId, owner_id: &AccountId, new_owner_id: &AccountId, amount: U128) -> TxResult {
        runtime.call(signer_id.clone(), self.contract_id.clone(), "transfer_from", json!({"owner_id": owner_id, "new_owner_id": new_owner_id, "amount": amount}), 0)
    }

    pub fn inc_allowance(&self, runtime: &mut TestRuntime, signer_id: &AccountId, escrow_account_id: AccountId, amount: U128) -> TxResult {
        runtime.call(signer_id.clone(), self.contract_id.clone(), "inc_allowance", json!({"escrow_account_id": escrow_account_id, "amount": amount}), 1024 * STORAGE_PRICE_PER_BYTE)
    }

    pub fn dec_allowance(&self, runtime: &mut TestRuntime, signer_id: &AccountId, escrow_account_id: AccountId, amount: U128) -> TxResult {
        runtime.call(signer_id.clone(), self.contract_id.clone(), "dec_allowance", json!({"escrow_account_id": escrow_account_id, "amount": amount}), 0)
    }

    pub fn get_total_supply(&self, runtime: &mut TestRuntime) -> String {
        runtime.view(self.contract_id.clone(), "get_total_supply", json!({})).as_str().unwrap().to_string()
    }

    pub fn get_balance(&self, runtime: &mut TestRuntime, owner_id: String) -> String {
        runtime.view(self.contract_id.clone(), "get_balance", json!({"owner_id": owner_id})).as_str().unwrap().to_string()
    }

    pub fn get_allowance(&self, runtime: &mut TestRuntime, owner_id: String, escrow_account_id: AccountId) -> String {
        runtime.view(self.contract_id.clone(), "get_balance", json!({"owner_id": owner_id, "escrow_account_id": escrow_account_id})).as_str().unwrap().to_string()
    }
}
