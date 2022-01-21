use near_sdk::{AccountId, Balance, PanicOnDefault, BorshStorageKey, log};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::{env, near_bindgen};
use near_sdk::serde::{Deserialize, Serialize};

pub use warrior::Warrior;
pub use battle::{Battle, EBattleConfig, InputError, parse_move, ParseError, make_actions};
pub use stats::{Stats, EStats};

mod warrior;
mod battle;
mod stats;

type BattleId = u64;

const BASE_STRENGTH: u16 = 1;
const BASE_STAMINA: u16 = 1;
const BASE_AGILITY: u16 = 1;
const BASE_INTUITION: u16 = 1;
const BASE_LIFE: u16 = 100;
const BASE_DEFENSE: u16 = 10;

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

    pub fn get_battle(&self, battle_id: &BattleId) -> Battle {
        self.battles.get(battle_id).expect("Battle not found")
    }

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

            let battle = Battle::new(account_id.clone(), account_id.clone(), None);

            self.battles.insert(&battle_id, &battle);
            // self.available_battles.insert(&battle_id, &(account_id.clone(), account_id.clone()));
            self.next_battle_id += 1;
            // self.available_warriors.remove(&account_id);
            self.add_referral(&account_id, &referrer_id);

            self.update_stats(&account_id, UpdateStatsAction::AddBattle, None, None);

            battle_id
        }
    }

    pub fn make_move(&mut self, battle_id: BattleId, params: String) {
        let mut battle: Battle = self.get_battle(&battle_id).into();
        assert!(battle.winner.is_none(), "Battle already finished");

        let mut update_battle = false;
        // let active_warrior = battle.current_warrior_account_id();
        // assert_eq!(active_warrior, env::predecessor_account_id(), "No access");

        // display::print_board(game.board());

        let parse_result = parse_move(&params);

        println!("Parse result: {:?}", parse_result);

        // TO DO: добавить ограничение по времени на ход
        match parse_result {
            Ok(actions) => {
                let log_message = format!("Actions: {:?}", actions);
                env::log(log_message.as_bytes());

                let result = make_actions(&mut battle, actions);
            }
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

        // match parse_result {
        //     Ok(positions) => {
        //         let move_result = util::apply_positions_as_move(&mut game, positions);
        //         match move_result {
        //             Ok(game_state) => match game_state {
        //                 GameState::InProgress => {
        //                     update_game = true;
        //                 }
        //                 GameState::GameOver { winner_id: winner_index } => {
        //                     let winner_account = game.players[winner_index].account_id.clone();
        //                     self.internal_distribute_reward(&game.reward, &winner_account);
        //                     game.winner_index = Some(winner_index);

        //                     self.internal_stop_game(game_id);

        //                     update_game = true;

        //                     log!("\nGame over! {} won!", winner_account);
        //                 }
        //             },
        //             Err(e) => match e {
        //                 MoveError::InvalidMove => panic!("\n *** Illegal move"),
        //                 MoveError::ShouldHaveJumped => panic!("\n *** Must take jump")
        //             }
        //         }
        //     }
        //     Err(e) => match e {
        //         InputError::TooFewTokens =>
        //             panic!("\n *** You must specify at least two board positions"),
        //         InputError::InvalidTokens { tokens: errors } => {
        //             for error in errors {
        //                 match error {
        //                     TokenError::MissingFile { token } =>
        //                         panic!("\n *** Board position '{}' must specify file", token),
        //                     TokenError::MissingRank { token } =>
        //                         panic!("\n *** Board position '{}' must specify rank", token),
        //                     TokenError::ZeroRank { token } =>
        //                         panic!("\n *** Rank cannot be zero: {}", token),
        //                     TokenError::InvalidCharacter { token, char_index } => {
        //                         let ch = token.chars().nth(char_index).unwrap();
        //                         panic!("\n *** Board position '{}' contains invalid character '{}'", token, ch);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }

        // if update_game {
        //     // display::print_board(game.board());
        //     game.turns += 1;
        //     let game_to_save: GameToSave = game.into();
        //     self.games.insert(&game_id, &game_to_save);
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use near_sdk::MockedBlockchain;
    // use near_sdk::testing_env;
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
}
