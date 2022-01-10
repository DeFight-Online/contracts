use near_sdk::{AccountId, Balance, PanicOnDefault, BorshStorageKey};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::{env, near_bindgen};
// use near_sdk::serde::{Deserialize, Serialize};

pub use warrior::Warrior;
pub use battle::{Battle, EBattleConfig};
pub use stats::{Stats, EStats};

mod warrior;
mod battle;
mod stats;

type BattleId = u64;

near_sdk::setup_alloc!();

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Battles,
    AvailableWarriors,
    Stats,
    AvailableBattles,
    Affiliates {account_id: AccountId},
    TotalRewards {account_id: AccountId},
    TotalAffiliateRewards{ account_id: AccountId},
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct DeFight {
    battles: LookupMap<BattleId, Battle>,
    available_warriors: UnorderedMap<AccountId, EBattleConfig>,
    stats: UnorderedMap<AccountId, EStats>,
    available_battles: UnorderedMap<BattleId, (AccountId, AccountId)>,
    next_battle_id: BattleId,
    service_fee: Balance,
}

#[near_bindgen]
impl DeFight {
    #[init]
    pub fn new() -> Self {
        Self {
            battles: LookupMap::new(StorageKey::Battles),
            available_warriors: UnorderedMap::new(StorageKey::AvailableWarriors),
            stats: UnorderedMap::new(StorageKey::Stats),
            available_battles: UnorderedMap::new(StorageKey::AvailableBattles),

            next_battle_id: 0,
            service_fee: 0,
        }
    }
}

#[near_bindgen]
impl DeFight {
    pub(crate) fn is_battle_started(&self, account_id: &AccountId) {
        let battles_already_started: Vec<(AccountId, AccountId)> = self.available_battles.values_as_vector()
            .iter()
            .filter(|(warrior_1, warrior_2)| *warrior_1 == *account_id || *warrior_2 == *account_id)
            .collect();
        assert_eq!(battles_already_started.len(), 0, "Another battle already started");
    }

    pub fn start_game(&mut self, opponent_id: Option<AccountId>, referrer_id: Option<AccountId>) -> BattleId {
        if let Some(opponent) = self.available_warriors.get(&opponent_id.unwrap_or("".to_string())) {
            panic!("PvP mode is not ready yet");
        } else {
            let account_id = env::predecessor_account_id();

            self.is_battle_started(&account_id);

            let battle_id = self.next_battle_id;
            battle_id
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::testing_env;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::json_types::ValidAccountId;
    use near_sdk::serde::export::TryFrom;

    // simple helper function to take a string literal and return a ValidAccountId
    fn to_valid_account(account: &str) -> ValidAccountId {
        ValidAccountId::try_from(account.to_string()).expect("Invalid account")
    }

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    // mark individual unit tests with #[test] for them to be registered and fired
    #[test]
    fn increment() {
        // set up the mock context into the testing environment
        let context = get_context(to_valid_account("foo.near"));
        testing_env!(context.build());
        // instantiate a contract variable with the counter at zero
        let mut contract = Counter { val: 0 };
        contract.increment();
        println!("Value after increment: {}", contract.get_num());
        // confirm that we received 1 when calling get_num
        assert_eq!(1, contract.get_num());
    }

    #[test]
    fn decrement() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract = Counter { val: 0 };
        contract.decrement();
        println!("Value after decrement: {}", contract.get_num());
        // confirm that we received -1 when calling get_num
        assert_eq!(-1, contract.get_num());
    }

    #[test]
    fn increment_and_reset() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract = Counter { val: 0 };
        contract.increment();
        contract.reset();
        println!("Value after reset: {}", contract.get_num());
        // confirm that we received -1 when calling get_num
        assert_eq!(0, contract.get_num());
    }
}
