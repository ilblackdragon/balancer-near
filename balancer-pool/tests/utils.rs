use near_crypto::{InMemorySigner, KeyType, Signer};
use near_primitives::{
    account::{AccessKey, Account},
    errors::{RuntimeError, TxExecutionError},
    hash::CryptoHash,
    transaction::{ExecutionOutcome, ExecutionStatus, Transaction},
    types::{AccountId, Balance},
};
use near_runtime_standalone::{init_runtime_and_signer, RuntimeStandalone};
use near_sdk::json_types::U128;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::convert::TryFrom;

const DEFAULT_GAS: u64 = 300_000_000_000_000;
const STORAGE_AMOUNT: u128 = 50_000_000_000_000_000_000_000_000;

lazy_static::lazy_static! {
    static ref TOKEN_WASM_BYTES: &'static [u8] = include_bytes!("../../test-token/res/test_token.wasm").as_ref();
    static ref POOL_WASM_BYTES: &'static [u8] = include_bytes!("../res/balancer_pool.wasm").as_ref();
}

type TxResult = Result<ExecutionOutcome, ExecutionOutcome>;

fn outcome_into_result(outcome: ExecutionOutcome) -> TxResult {
    match outcome.status {
        ExecutionStatus::SuccessValue(_) => Ok(outcome),
        ExecutionStatus::Failure(_) => Err(outcome),
        ExecutionStatus::SuccessReceiptId(_) => panic!("Unresolved ExecutionOutcome run runtime.resolve(tx) to resolve the final outcome of tx"),
        ExecutionStatus::Unknown => unreachable!()
    }
}

fn to_yocto(value: &str) -> u128 {
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

pub struct ExternalUser {
    runtime: RuntimeStandalone,
    pub account_id: AccountId,
    signer: InMemorySigner,
}

impl ExternalUser {
    pub fn new(runtime: RuntimeStandalone, account_id: AccountId, signer: InMemorySigner) -> Self {
        Self { runtime, account_id, signer }
    }

    fn transaction(&self, receiver_id: AccountId) -> Transaction {
        let nonce = self.runtime.view_access_key(&self.account_id, &self.signer.public_key()).unwrap().nonce + 1;
        Transaction::new(self.account_id.clone(), self.signer.public_key(), receiver_id, nonce, CryptoHash::default())
    }

    fn submit_transaction(&mut self, transaction: Transaction) -> TxResult {
        let res = self.runtime.resolve_tx(transaction.sign(&self.signer)).unwrap();
        self.runtime.process_all().unwrap();
        outcome_into_result(res)
    }

    pub fn deploy(&mut self, contract_id: AccountId, wasm_bytes: &[u8], args: serde_json::Value) -> TxResult {
        self.submit_transaction(
            self.transaction(contract_id).create_account().transfer(STORAGE_AMOUNT).deploy_contract(wasm_bytes.to_vec()).function_call("new".to_string(), args.to_string().as_bytes().to_vec(), DEFAULT_GAS, 0))
    }

    pub fn call(&mut self, contract_id: AccountId, method: &str, args: serde_json::Value, deposit: u128) -> TxResult {
        println!("{:?}", args.to_string());
        self.submit_transaction(self.transaction(contract_id).function_call(method.to_string(), args.to_string().as_bytes().to_vec(), DEFAULT_GAS, deposit))
    }

    pub fn view(&mut self, contract_id: AccountId, method: &str, args: serde_json::Value) -> serde_json::Value {
        serde_json::from_slice(&self.runtime.view_method_call(&contract_id, &method.to_string(), args.to_string().as_bytes()).unwrap().0).unwrap()
    }
}

pub fn init_user() -> ExternalUser {
    let (mut runtime, signer) = init_runtime_and_signer(&"root".into());
    ExternalUser::new(runtime, "root".into(), signer)
}

pub struct TToken {
    pub contract_id: AccountId,
}

impl TToken {
    pub fn new(user: &mut ExternalUser, contract_id: AccountId, owner_id: &AccountId, total_supply: &str) -> Self {
        let _ = user.deploy(contract_id.clone(), &TOKEN_WASM_BYTES, json!({"owner_id": user.account_id, "total_supply": U128::from(to_yocto(total_supply))})).unwrap();
        Self { contract_id }
    }

    pub fn mint(&self, user: &mut ExternalUser, account_id: &AccountId, amount: &str) {
        let _ = user.call(self.contract_id.clone(), "mint", json!({"account_id": account_id, "amount": U128::from(to_yocto(amount))}), 0).unwrap();
    }
}

pub struct BPool {
    contract_id: AccountId,
}

impl BPool {
    pub fn new(user: &mut ExternalUser, contract_id: AccountId) -> Self {
        let _ = user.deploy(contract_id.clone(), &POOL_WASM_BYTES, json!({})).unwrap();
        Self { contract_id }
    }

    pub fn getController(&self, user: &mut ExternalUser) -> AccountId {
        user.view(self.contract_id.clone(), "getController", json!({})).as_str().unwrap().to_string()
    }

    pub fn getNumTokens(&self, user: &mut ExternalUser) -> u64 {
        user.view(self.contract_id.clone(), "getNumTokens", json!({})).as_u64().unwrap()
    }

    pub fn bind(&self, user: &mut ExternalUser, token: AccountId, balance: &str, denorm: &str) {
        let _ = user.call(self.contract_id.clone(), "bind", json!({"token": token, "balance": U128::from(to_yocto(balance)), "denorm": U128::from(to_yocto(denorm))}), 0).unwrap();
    }
}
