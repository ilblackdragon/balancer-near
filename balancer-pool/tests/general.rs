mod utils;

use utils::{init_user, ExternalUser, TToken, BPool};

const WETH: &str = "weth";
const MKR: &str = "mkr";
const DAI: &str = "dai";
const XXX: &str = "xxx";
const POOL: &str = "pool";

fn setup_multi_token_pool() -> (ExternalUser, BPool, TToken, TToken, TToken, TToken) {
    let mut user = init_user();
    let root = user.account_id.clone();
    let user1 = "user1".to_string();
    let user2 = "user2".to_string();

    let pool = BPool::new(&mut user, POOL.to_string());

    let weth = TToken::new(&mut user, WETH.to_string(), &root, "50");
    let mkr = TToken::new(&mut user, MKR.to_string(), &root, "20");
    let dai = TToken::new(&mut user, DAI.to_string(), &root, "10000");
    let xxx = TToken::new(&mut user, XXX.to_string(), &root, "10");

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