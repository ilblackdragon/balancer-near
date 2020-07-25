use std::convert::TryFrom;

use near_sdk::json_types::U128;
use near_sdk::AccountId;

use near_crypto::{InMemorySigner, KeyType, Signer};
use near_primitives::{
    account::{AccessKey, Account},
    errors::{RuntimeError, TxExecutionError},
    hash::CryptoHash,
    transaction::{ExecutionOutcome, ExecutionStatus, Transaction},
    types::{AccountId, Balance},
};
use near_runtime_standalone::{init_runtime_and_signer, RuntimeStandalone};
use serde::de::DeserializeOwned;
use serde_json::json;

const DEFAULT_GAS: u64 = 300_000_000_000_000;
const STORAGE_AMOUNT: u128 = 50_000_000_000_000_000_000_000_000;

type TxResult = Result<ExecutionOutcome, ExecutionOutcome>;

fn outcome_into_result(outcome: ExecutionOutcome) -> TxResult {
    match outcome.status {
        ExecutionStatus::SuccessValue(_) => Ok(outcome),
        ExecutionStatus::Failure(_) => Err(outcome),
        ExecutionStatus::SuccessReceiptId(_) => panic!("Unresolved ExecutionOutcome run runtime.resolve(tx) to resolve the final outcome of tx"),
        ExecutionStatus::Unknown => unreachable!()
    }
}

pub fn to_yocto(value: &str) -> u128 {
    let vals: Vec<_> = value.split(".").collect();
    let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
    if vals.len() > 1 {
        let power = vals[1].len() as u32;
        let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
        part1 + part2
    } else {
        part1
    }
}

pub struct TestUser<'a> {
    runtime: &'a RuntimeStandalone,
    pub account_id: AccountId,
    signer: InMemorySigner,
}

impl TestUser {
    pub fn new(
        runtime: &'a RuntimeStandalone,
        account_id: AccountId,
        signer: InMemorySigner,
    ) -> Self {
        Self {
            runtime,
            account_id,
            signer,
        }
    }

    fn transaction(&self, receiver_id: AccountId) -> Transaction {
        let nonce = self
            .runtime
            .view_access_key(&self.account_id, &self.signer.public_key())
            .unwrap()
            .nonce
            + 1;
        Transaction::new(
            self.account_id.clone(),
            self.signer.public_key(),
            receiver_id,
            nonce,
            CryptoHash::default(),
        )
    }

    fn submit_transaction(&mut self, transaction: Transaction) -> TxResult {
        let res = self
            .runtime
            .resolve_tx(transaction.sign(&self.signer))
            .unwrap();
        self.runtime.process_all().unwrap();
        outcome_into_result(res)
    }

    pub fn deploy(
        &mut self,
        contract_id: AccountId,
        wasm_bytes: &[u8],
        args: serde_json::Value,
    ) -> TxResult {
        self.submit_transaction(
            self.transaction(contract_id)
                .create_account()
                .transfer(STORAGE_AMOUNT)
                .deploy_contract(wasm_bytes.to_vec())
                .function_call(
                    "new".to_string(),
                    args.to_string().as_bytes().to_vec(),
                    DEFAULT_GAS,
                    0,
                ),
        )
    }

    pub fn call(
        &mut self,
        contract_id: AccountId,
        method: &str,
        args: serde_json::Value,
        deposit: u128,
    ) -> TxResult {
        self.submit_transaction(self.transaction(contract_id).function_call(
            method.to_string(),
            args.to_string().as_bytes().to_vec(),
            DEFAULT_GAS,
            deposit,
        ))
    }

    pub fn view(
        &mut self,
        contract_id: AccountId,
        method: &str,
        args: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::from_slice(
            &self
                .runtime
                .view_method_call(
                    &contract_id,
                    &method.to_string(),
                    args.to_string().as_bytes(),
                )
                .unwrap()
                .0,
        )
        .unwrap()
    }

    pub fn create_user(&mut self, account_id: AccountId, amount: Balance) -> TestUser {
        self.submit_transaction(
            self.transaction(account_id)
                .create_account()
                .transfer(amount),
        )
        .expect();
        TestUser::new(self.runtime, account_id, self.signer.clone())
    }
}

pub fn init_test_user() -> TestUser {
    let (mut runtime, signer) = init_runtime_and_signer(&"root".into());
    TestUser::new(runtime, "root".into(), signer)
}
