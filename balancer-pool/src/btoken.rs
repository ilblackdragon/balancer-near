use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, AccountId, Balance, Promise, StorageUsage, ext_contract};

/// Contains balance and allowances information for one account.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    /// Current account balance.
    pub balance: Balance,
    /// Escrow Account ID hash to the allowance amount.
    /// Allowance is the amount of tokens the Escrow Account ID can spent on behalf of the account
    /// owner.
    pub allowances: UnorderedMap<Vec<u8>, Balance>,
}

impl Account {
    /// Initializes a new Account with 0 balance and no allowances for a given `account_hash`.
    pub fn new(account_hash: Vec<u8>) -> Self {
        Self { balance: 0, allowances: UnorderedMap::new(account_hash) }
    }

    /// Sets allowance for account `escrow_account_id` to `allowance`.
    pub fn set_allowance(&mut self, escrow_account_id: &AccountId, allowance: Balance) {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        if allowance > 0 {
            self.allowances.insert(&escrow_hash, &allowance);
        } else {
            self.allowances.remove(&escrow_hash);
        }
    }

    /// Returns the allowance of account `escrow_account_id`.
    pub fn get_allowance(&self, escrow_account_id: &AccountId) -> Balance {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        self.allowances.get(&escrow_hash).unwrap_or(0)
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct BToken {
    name: String,
    symbol: String,
    owner: AccountId,
    total_supply: Balance,

    accounts: UnorderedMap<Vec<u8>, Account>,
}

impl BToken {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            symbol: "".to_string(),
            owner: env::predecessor_account_id(),
            total_supply: 0,
            accounts: UnorderedMap::new(b"a".to_vec())
        }
    }
    pub fn mint(&mut self, amount: Balance) {
        let mut account = self.get_account(&env::current_account_id());
        account.balance += amount;
        self.total_supply += amount;
        self.set_account(&env::current_account_id(), &account);
    }

    pub fn burn(&mut self, amount: Balance) {
        let mut account = self.get_account(&env::current_account_id());
        assert!(account.balance >= amount, "ERR_INSUFFICIENT_BAL");
        account.balance -= amount;
        self.total_supply -= amount;
        self.set_account(&env::current_account_id(), &account);
    }

    pub fn transfer_from(&mut self, from: &AccountId, to: &AccountId, amount: Balance) {
        let mut from_account = self.get_account(from);
        assert!(from_account.balance >= amount, "ERR_INSUFFICIENT_BAL");
        let mut to_account = self.get_account(to);
        from_account.balance -= amount;
        to_account.balance += amount;
        self.set_account(from, &from_account);
        self.set_account(to, &to_account);
    }

    pub fn push(&mut self, to: &AccountId, amount: Balance) {
        self.transfer_from(&env::current_account_id(), to, amount)
    }

    pub fn pull(&mut self, from: &AccountId, amount: Balance) {
        self.transfer_from(from, &env::current_account_id(), amount);
    }

    #[inline]
    pub fn total_supply(&self) -> Balance {
        self.total_supply
    }

    pub fn balance(&self, account_id: AccountId) -> Balance {
        self.get_account(&account_id).balance
    }

    /// Helper method to get the account details for `owner_id`.
    fn get_account(&self, owner_id: &AccountId) -> Account {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Owner's account ID is invalid");
        let account_hash = env::sha256(owner_id.as_bytes());
        self.accounts.get(&account_hash).unwrap_or_else(|| Account::new(account_hash))
    }

    /// Helper method to set the account details for `owner_id` to the state.
    fn set_account(&mut self, owner_id: &AccountId, account: &Account) {
        let account_hash = env::sha256(owner_id.as_bytes());
        if account.balance > 0 || !account.allowances.is_empty() {
            self.accounts.insert(&account_hash, &account);
        } else {
            self.accounts.remove(&account_hash);
        }
    }
}

/// Trait with FungibleToken interface that implements NEP-21 standard.
pub trait FungibleToken {
    // TODO: ??
}

#[ext_contract(ext_nep21)]
pub trait ExtNEP21 {
    fn transfer(&mut self, new_owner_id: AccountId, amount: U128);

    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: U128);

    fn balance(&self, account_id: AccountId) -> U128;
}
