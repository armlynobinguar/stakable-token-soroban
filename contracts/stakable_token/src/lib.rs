#![no_std]
use core::convert::TryInto;
use soroban_sdk::{contract, contractimpl, map, symbol_short, Address, Env, Map, Symbol};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const BALANCES_KEY: Symbol = symbol_short!("BAL");
const STAKED_AMOUNT_KEY: Symbol = symbol_short!("STAKED");
const STAKED_AT_KEY: Symbol = symbol_short!("STAKED_AT");
const ONE_YEAR_SECONDS: u64 = 365 * 24 * 3600;
const REWARD_BPS: u64 = 1000; // 10% = 1000 basis points

#[contract]
pub struct StakableToken;

#[contractimpl]
impl StakableToken {
    pub fn init(env: Env, admin: Address) {
        env.storage().instance().set(&ADMIN_KEY, &admin);
        let balances: Map<Address, i128> = map![&env];
        env.storage().instance().set(&BALANCES_KEY, &balances);
    }

    fn read_admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("not initialized")
    }

    fn read_balances(env: &Env) -> Map<Address, i128> {
        env.storage()
            .instance()
            .get(&BALANCES_KEY)
            .unwrap_or_else(|| map![env])
    }

    pub fn balance(env: Env, addr: Address) -> i128 {
        let balances = Self::read_balances(&env);
        balances.get(addr).unwrap_or(0)
    }

    pub fn staked_balance(env: Env, addr: Address) -> i128 {
        let key = (STAKED_AMOUNT_KEY, addr);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) {
        let stored_admin = Self::read_admin(&env);
        admin.require_auth();
        assert!(admin == stored_admin, "only admin can mint");

        let mut balances = Self::read_balances(&env);
        let current = balances.get(to.clone()).unwrap_or(0);
        balances.set(to, current + amount);
        env.storage().instance().set(&BALANCES_KEY, &balances);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        let mut balances = Self::read_balances(&env);
        let from_bal = balances.get(from.clone()).unwrap_or(0);
        let to_bal = balances.get(to.clone()).unwrap_or(0);

        assert!(from_bal >= amount, "insufficient");

        balances.set(from.clone(), from_bal - amount);
        balances.set(to, to_bal + amount);
        env.storage().instance().set(&BALANCES_KEY, &balances);
    }

    /// Stake tokens. Locks balance and starts earning rewards.
    pub fn stake(env: Env, from: Address, amount: i128) {
        from.require_auth();
        assert!(amount > 0, "amount must be positive");

        let mut balances = Self::read_balances(&env);
        let from_bal = balances.get(from.clone()).unwrap_or(0);
        assert!(from_bal >= amount, "insufficient balance");

        let now = env.ledger().timestamp();
        let key_amt = (STAKED_AMOUNT_KEY, from.clone());
        let key_at = (STAKED_AT_KEY, from.clone());

        let existing_staked: i128 = env.storage().instance().get(&key_amt).unwrap_or(0);
        let existing_at: u64 = env.storage().instance().get(&key_at).unwrap_or(now);

        // New staked total and weighted start time for reward calc
        let new_staked = existing_staked + amount;
        let new_at = if existing_staked == 0 {
            now
        } else {
            ((existing_at as i128 * existing_staked + now as i128 * amount) / new_staked)
                .try_into()
                .unwrap()
        };

        balances.set(from.clone(), from_bal - amount);
        env.storage().instance().set(&BALANCES_KEY, &balances);
        env.storage().instance().set(&key_amt, &new_staked);
        env.storage().instance().set(&key_at, &new_at);
    }

    /// Unstake tokens and claim rewards. Rewards are paid from contract's balance (admin must fund).
    pub fn unstake(env: Env, from: Address, amount: i128) {
        from.require_auth();
        assert!(amount > 0, "amount must be positive");

        let key_amt = (STAKED_AMOUNT_KEY, from.clone());
        let key_at = (STAKED_AT_KEY, from.clone());
        let staked: i128 = env.storage().instance().get(&key_amt).unwrap_or(0);
        let staked_at: u64 = env.storage().instance().get(&key_at).unwrap_or(0);

        assert!(staked >= amount, "insufficient staked");

        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(staked_at);
        // reward = amount * (elapsed / ONE_YEAR) * (REWARD_BPS / 10000)
        let reward = (amount as u64)
            .saturating_mul(elapsed)
            .saturating_mul(REWARD_BPS)
            .checked_div(ONE_YEAR_SECONDS.saturating_mul(10_000))
            .unwrap_or(0) as i128;

        let new_staked = staked - amount;
        let mut balances = Self::read_balances(&env);
        let user_bal = balances.get(from.clone()).unwrap_or(0);
        let contract_addr = env.current_contract_address();
        let pool_bal = balances.get(contract_addr.clone()).unwrap_or(0);
        assert!(pool_bal >= reward, "insufficient reward pool");
        balances.set(contract_addr, pool_bal - reward);
        balances.set(from.clone(), user_bal + amount + reward);

        if new_staked == 0 {
            env.storage().instance().remove(&key_amt);
            env.storage().instance().remove(&key_at);
        } else {
            env.storage().instance().set(&key_amt, &new_staked);
            env.storage().instance().set(&key_at, &now);
        }
        env.storage().instance().set(&BALANCES_KEY, &balances);
    }

    /// View pending reward for an address (estimate based on current time).
    pub fn pending_reward(env: Env, addr: Address) -> i128 {
        let key_amt = (STAKED_AMOUNT_KEY, addr.clone());
        let key_at = (STAKED_AT_KEY, addr);
        let staked: i128 = env.storage().instance().get(&key_amt).unwrap_or(0);
        let staked_at: u64 = env.storage().instance().get(&key_at).unwrap_or(0);
        if staked == 0 {
            return 0;
        }
        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(staked_at);
        (staked as u64)
            .saturating_mul(elapsed)
            .saturating_mul(REWARD_BPS)
            .checked_div(ONE_YEAR_SECONDS.saturating_mul(10_000))
            .unwrap_or(0) as i128
    }
}

#[cfg(test)]
mod test;
