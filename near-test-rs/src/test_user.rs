use near_sdk::AccountId;

use near_crypto::{InMemorySigner, Signer};
use near_primitives::{
    hash::CryptoHash,
    transaction::{ExecutionOutcome, ExecutionStatus, Transaction},
    types::{Balance},
};
use near_runtime_standalone::{init_runtime_and_signer, RuntimeStandalone};

const DEFAULT_GAS: u64 = 300_000_000_000_000;
const STORAGE_AMOUNT: u128 = 50_000_000_000_000_000_000_000_000;

pub type TxResult = Result<ExecutionOutcome, ExecutionOutcome>;

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

pub struct TestRuntime {
    runtime: RuntimeStandalone,
    signer: InMemorySigner,
}

impl TestRuntime {
    pub fn new(
        runtime: RuntimeStandalone,
        signer: InMemorySigner,
    ) -> Self {
        Self {
            runtime,
            signer,
        }
    }

    fn transaction(&self, signer_id: AccountId, receiver_id: AccountId) -> Transaction {
        let nonce = self
            .runtime
            .view_access_key(&signer_id, &self.signer.public_key())
            .unwrap()
            .nonce
            + 1;
        Transaction::new(
            signer_id.clone(),
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
        signer_id: AccountId,
        contract_id: AccountId,
        wasm_bytes: &[u8],
        args: serde_json::Value,
    ) -> TxResult {
        self.submit_transaction(
            self.transaction(signer_id, contract_id)
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
        signer_id: AccountId,
        contract_id: AccountId,
        method: &str,
        args: serde_json::Value,
        deposit: u128,
    ) -> TxResult {
        self.call(signer_id, contract_id, method, args.to_string().as_bytes().to_vec(), deposit)
   }

    pub fn call_args(&mut self, signer_id: AccountId, contract_id: AccountId, method: &str, args: Vec<u8>, depsit: u128) -> TxResult {
       self.submit_transaction(self.transaction(signer_id, contract_id).function_call(
            method.to_string(),
            args,
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

    pub fn create_user(&mut self, signer_id: AccountId, account_id: AccountId, amount: Balance) {
        self.submit_transaction(
            self.transaction(signer_id, account_id)
                .create_account()
                .transfer(amount),
        )
        .unwrap();
    }
}

pub fn init_test_runtime() -> TestRuntime {
    let (runtime, signer) = init_runtime_and_signer(&"root".into());
    TestRuntime::new(runtime, signer)
}
