use near_sdk::{AccountId, Balance, PanicOnDefault, BorshStorageKey, log, Timestamp, PromiseResult};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::{env, near_bindgen};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;
use near_contract_standards::non_fungible_token::{Token, TokenId, NonFungibleToken};
// use near_contract_standards::non_fungible_token::metadata::{
//   NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
// };

pub use warrior::Warrior;
pub use battle::{Battle, BattleToSave, EBattleConfig, InputError, parse_move, ParseError, BattleState};
pub use stats::{Stats, EStats};
pub use crate::callbacks::*;

mod warrior;
mod battle;
mod stats;
mod callbacks;

type BattleId = u64;

const BASE_STRENGTH: u16 = 1;
const BASE_STAMINA: u16 = 1;
const BASE_AGILITY: u16 = 1;
const BASE_INTUITION: u16 = 1;
const BASE_HEALTH: u16 = 10;
const BASE_DEFENSE: u16 = 1;

const MAX_MS_FOR_ACTION: u64 = 60_000_000_000;

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

#[derive(PartialEq)]
pub enum UpdateStatsAction {
    AddBattle,
    AddReferral,
    AddAffiliate,
    AddWonBattle,
    AddLostBattle,
    AddTotalReward,
    AddAffiliateReward,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct DeFight {
    battles: LookupMap<BattleId, BattleToSave>,
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
    #[result_serializer(borsh)]
    pub fn resolve_get(
        &mut self,
    ) {
        let log_message = format!("Cross-contract callback");
        env::log(log_message.as_bytes());

        env::log(log_message.as_bytes()); 
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic(b"Unable to get user tokens"),
            PromiseResult::Successful(result) => {
                // let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                let log_message = format!("User tokens: {:?}", result);
                env::log(log_message.as_bytes());
            },
          }
    }

    pub(crate) fn is_account_exists(&self, account_id: &Option<AccountId>) -> bool {
        if let Some(account_id_unwrapped) = account_id {
            self.stats.get(account_id_unwrapped).is_some()
        } else {
            false
        }
    }

    pub(crate) fn get_stats(&self, account_id: &AccountId) -> Stats {
        if let Some(stats) = self.stats.get(account_id) {
            stats.into()
        } else {
            Stats::new(&account_id)
        }
    }

    pub(crate) fn update_stats(&mut self,
        account_id: &AccountId,
        action: UpdateStatsAction,
        additional_account_id: Option<AccountId>,
        balance: Option<Balance>,
    ) {
        let mut stats = self.get_stats(account_id);

        if action == UpdateStatsAction::AddBattle {
            stats.battles_num += 1
        } else if action == UpdateStatsAction::AddReferral {
            if additional_account_id.is_some() {
                stats.referrer_id = additional_account_id;
            }
        } else if action == UpdateStatsAction::AddAffiliate {
            if let Some(additional_account_id_unwrapped) = additional_account_id {
                stats.affiliates.insert(&additional_account_id_unwrapped);
            }
        } else if action == UpdateStatsAction::AddWonBattle {
            stats.wins_num += 1;
        } else if action == UpdateStatsAction::AddTotalReward {
            if let Some(balance_unwrapped) = balance {
                let total_reward = stats.total_reward.get(&None).unwrap_or(0);
                stats.total_reward.insert(&None, &(total_reward + balance_unwrapped));
            }
        } else if action == UpdateStatsAction::AddAffiliateReward {
            if let Some(balance_unwrapped) = balance {
                // TODO Add FT
                let total_affiliate_reward = stats.total_affiliate_reward.get(&None).unwrap_or(0);
                stats.total_affiliate_reward.insert(&None, &(total_affiliate_reward + balance_unwrapped));
            }
        }

        self.stats.insert(account_id, &EStats::Current(stats));
    }

    pub(crate) fn add_referral(&mut self, account_id: &AccountId, referrer_id: &Option<AccountId>) {
        if self.stats.get(account_id).is_none() && self.is_account_exists(referrer_id) {
            if let Some(referrer_id_unwrapped) = referrer_id.clone() {
                self.update_stats(account_id, UpdateStatsAction::AddReferral, referrer_id.clone(), None);
                self.update_stats(&referrer_id_unwrapped, UpdateStatsAction::AddAffiliate, Some(account_id.clone()), None);
                log!("Referrer {} added for {}", referrer_id_unwrapped, account_id);
            }
        }
    }

    pub fn get_battle(&self, battle_id: &BattleId) -> BattleToSave {
        let battle = self.battles.get(battle_id).expect("Battle not found");

        let log_message = format!("Battle state: {:?}", battle);
        env::log(log_message.as_bytes());

        battle
    }

    pub(crate) fn is_battle_started(&self, account_id: &AccountId) {
        let battles_already_started: Vec<(AccountId, AccountId)> = self.available_battles.values_as_vector()
            .iter()
            .filter(|(warrior_1, warrior_2)| *warrior_1 == *account_id || *warrior_2 == *account_id)
            .collect();
        assert_eq!(battles_already_started.len(), 0, "Another battle already started");
    }

    pub fn start_battle(&mut self, opponent_id: Option<AccountId>, referrer_id: Option<AccountId>) -> BattleId {
        if let Some(_opponent) = self.available_warriors.get(&opponent_id.unwrap_or("".to_string())) {
            panic!("PvP mode is not ready yet");
        } else {
            let account_id = env::predecessor_account_id();

            self.is_battle_started(&account_id);

            let battle_id = self.next_battle_id;

            let battle = BattleToSave::new(account_id.clone(), account_id.clone(), None);

            self.battles.insert(&battle_id, &battle);
            self.next_battle_id += 1;
        
            self.add_referral(&account_id, &referrer_id);
            self.update_stats(&account_id, UpdateStatsAction::AddBattle, None, None);

            battle_id
        }
    }

    #[result_serializer(borsh)]
    pub fn make_action(&mut self, battle_id: BattleId, params: String) {
        let mut battle: Battle = self.get_battle(&battle_id).into();
        
        assert!(battle.winner.is_none(), "Battle has already finished");

        // const account_id: String = env::predecessor_account_id();
        // Initiating receiver's call and the callback
        ext_paras_receiver::nft_tokens_for_owner(
            env::signer_account_id(),
            None,
            None,
            &"paras-token-v2.testnet".to_string(), //contract account to make the call to
            0, //attached deposit
            5_000_000_000_000,
        )

        //we then resolve the promise and call nft_resolve_transfer on our own contract
        .then(ext_self::resolve_get(
            &env::current_account_id(), //contract account to make the call to
            0, //attached deposit
            5_000_000_000_000, //GAS attached to the call
        ));

        let log_message = format!("Battle state: {:?}", battle.winner.is_none());
        env::log(log_message.as_bytes());

        let parse_result = parse_move(&params);

        match parse_result {
            Ok(actions) => {
                let log_message = format!("Actions: {:?}", actions);
                env::log(log_message.as_bytes());

                let result = battle.apply_actions(actions);

                let log_message = format!("Result: {:?}", result);
                env::log(log_message.as_bytes());   
                self.battles.insert(&battle_id, &result);

                if result.winner == Some(0) {
                    let log_message = format!("Battle is over! Draw");
                    env::log(log_message.as_bytes());  
                }

                if result.winner == Some(1) {
                    let log_message = format!("Battle is over! Winner: {:?}", result.warrior_1.account_id);
                    env::log(log_message.as_bytes());  
                }

                if result.winner == Some(2) {
                    let log_message = format!("Battle is over! Winner: {:?}", result.warrior_2.account_id);
                    env::log(log_message.as_bytes());  
                }
            },
            Err(e) => match e {
                InputError::WrongActions { actions: errors } => {
                    for error in errors {
                        match error {
                            ParseError::WrongAction { action } =>
                                panic!("\n *** Action {} doesn't exist in the game", action),
                            ParseError::WrongPart { part } =>
                                panic!("\n *** Part '{}' doesn't exist in the game", part),
                        }
                    }
                }
                InputError::TooFewActions =>
                    panic!("\n *** You must specify two actions - Attack and Protect"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
}