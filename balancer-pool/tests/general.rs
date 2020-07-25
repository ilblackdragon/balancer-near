use near_sdk::json_types::U128;
use near_sdk::AccountId;
use serde_json::json;

use near_lib::test_user::{init_test_user, to_yocto, TestUser};
use near_lib::token::test::TokenContract;

const WETH: &str = "weth";
const MKR: &str = "mkr";
const DAI: &str = "dai";
const XXX: &str = "xxx";
const POOL: &str = "pool";

lazy_static::lazy_static! {
    static ref TOKEN_WASM_BYTES: &'static [u8] = include_bytes!("../../test-token/res/test_token.wasm").as_ref();
    static ref POOL_WASM_BYTES: &'static [u8] = include_bytes!("../res/balancer_pool.wasm").as_ref();
}

pub struct BPool {
    contract_id: AccountId,
}

impl BPool {
    pub fn new(user: &mut TestUser, contract_id: AccountId) -> Self {
        let _ = user
            .deploy(contract_id.clone(), &POOL_WASM_BYTES, json!({}))
            .unwrap();
        Self { contract_id }
    }

    pub fn getController(&self, user: &mut TestUser) -> AccountId {
        user.view(self.contract_id.clone(), "getController", json!({}))
            .as_str()
            .unwrap()
            .to_string()
    }

    pub fn getNumTokens(&self, user: &mut TestUser) -> u64 {
        user.view(self.contract_id.clone(), "getNumTokens", json!({}))
            .as_u64()
            .unwrap()
    }

    pub fn bind(&self, user: &mut TestUser, token: AccountId, balance: &str, denorm: &str) {
        let _ = user.call(self.contract_id.clone(), "bind", json!({"token": token, "balance": U128::from(to_yocto(balance)), "denorm": U128::from(to_yocto(denorm))}), 0).unwrap();
    }
}

fn setup_multi_token_pool() -> (
    TestUser,
    BPool,
    TokenContract,
    TokenContract,
    TokenContract,
    TokenContract,
) {
    let mut user = init_test_user();
    let root = user.account_id.clone();
    let user1 = "user1".to_string();
    let user2 = "user2".to_string();

    let pool = BPool::new(&mut user, POOL.to_string());

    let weth = TokenContract::new(&mut user, &TOKEN_WASM_BYTES, WETH.to_string(), &root, "50");
    let mkr = TokenContract::new(&mut user, &TOKEN_WASM_BYTES, MKR.to_string(), &root, "20");
    let dai = TokenContract::new(
        &mut user,
        &TOKEN_WASM_BYTES,
        DAI.to_string(),
        &root,
        "10000",
    );
    let xxx = TokenContract::new(&mut user, &TOKEN_WASM_BYTES, XXX.to_string(), &root, "10");

    // User1 balances.
    weth.mint(&mut user, &user1, "25");
    mkr.mint(&mut user, &user1, "4");
    dai.mint(&mut user, &user1, "40000");
    xxx.mint(&mut user, &user1, "10");

    // User2 balances.
    weth.mint(&mut user, &user2, "12.2222");
    mkr.mint(&mut user, &user2, "1.015333");
    dai.mint(&mut user, &user2, "0");
    xxx.mint(&mut user, &user2, "51");

    (user, pool, weth, mkr, dai, xxx)
}

#[test]
fn multi_token_pool() {
    let (mut user, pool, weth, mkr, dai, xxx) = setup_multi_token_pool();
    let root = user.account_id.clone();
    assert_eq!(pool.getController(&mut user), root);
    assert_eq!(pool.getNumTokens(&mut user), 0);
}

#[test]
fn deposit_failure() {
    let (mut user, pool, weth, mkr, dai, xxx) = setup_multi_token_pool();
    pool.bind(&mut user, weth.contract_id, "100", "1");
}
