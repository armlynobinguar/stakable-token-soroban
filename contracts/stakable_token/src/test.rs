#![cfg(test)]
use crate::{StakableToken, StakableTokenClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
fn test_stake_and_unstake() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register(StakableToken, ());
    let client = StakableTokenClient::new(&env, &contract_id);
    client.init(&admin);

    // Admin mints to user and to contract (reward pool)
    client.mint(&admin, &user, &1000);
    client.mint(&admin, &contract_id, &100_000);

    assert_eq!(client.balance(&user), 1000);
    assert_eq!(client.staked_balance(&user), 0);

    client.stake(&user, &500);
    assert_eq!(client.balance(&user), 500);
    assert_eq!(client.staked_balance(&user), 500);

    // Unstake (no time advance, so reward is 0). User gets 500 back → total balance 500 + 500 = 1000
    client.unstake(&user, &500);
    assert_eq!(client.staked_balance(&user), 0);
    assert_eq!(client.balance(&user), 1000);
}
