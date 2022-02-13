use crate::*;
use lazy_static::lazy_static;
use strum::{EnumVariantNames, VariantNames};
use regex::Regex;
use std::str::FromStr;
use near_sdk::env::random_seed;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Battle {
    pub warrior_1: Warrior,
    pub warrior_2: Warrior,
    pub winner: Option<u32>,
    pub reward: Balance,
    pub last_action_timestamp: Timestamp,
    pub warrior_1_missed_action: bool,
    pub warrior_2_missed_action: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct BattleConfig {
    pub(crate) deposit: Option<Balance>,
    pub(crate) opponent_id: Option<AccountId>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum EBattleConfig {
    Current(BattleConfig)
}

impl From<EBattleConfig> for BattleConfig {
    fn from(e_battle_config: EBattleConfig) -> Self {
        match e_battle_config {
            EBattleConfig::Current(battle_config) => battle_config,
        }
    }
}

#[derive(PartialEq, EnumVariantNames, Debug, Copy, Clone)]
pub enum Part {
    Head,
    Neck,
    Chest,
    Groin,
    Legs,
}

impl FromStr for Part {
    type Err = ();

    fn from_str(input: &str) -> Result<Part, Self::Err> {
        match input {
            "Head" => Ok(Part::Head),
            "Neck" => Ok(Part::Neck),
            "Chest" => Ok(Part::Chest),
            "Groin" => Ok(Part::Groin),
            "Legs" => Ok(Part::Legs),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum InputError {
    WrongActions { actions : Vec<ParseError> },
    TooFewActions,
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    WrongAction { action : String },
    WrongPart { part : String },
}

#[derive(PartialEq, EnumVariantNames, Debug, Copy, Clone)]
pub enum ActionType {
    Attack,
    Protect,
}

impl FromStr for ActionType {
    type Err = ();

    fn from_str(input: &str) -> Result<ActionType, Self::Err> {
        match input {
            "Attack" => Ok(ActionType::Attack),
            "Protect" => Ok(ActionType::Protect),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct MoveData {
    action: ActionType,
    part: Part,
}

impl MoveData {
    pub fn new(action: ActionType, part: Part) -> MoveData {
        MoveData { action, part }
    }
}


lazy_static! {
    // A Regular Expression used to find variant names in target strings. 
    static ref ACTION_EXPR: Regex = {
        // Piece together the expression from Thing's variant names.
        let expr_str = ActionType::VARIANTS.join("|");

        Regex::new(&expr_str).unwrap()  
    };

    static ref PART_EXPR: Regex = {    
        // Piece together the expression from Thing's variant names.
        let expr_str = Part::VARIANTS.join("|");
        
        Regex::new(&expr_str).unwrap()  
    };
}

fn part_validator(string_to_validate: &str) -> Result<MoveData, ParseError> {
	let result: Vec<&str> = string_to_validate.split(":").collect();

    if ACTION_EXPR.captures(result[0]).is_none() {
        
        return Err(ParseError::WrongAction { action : result[0].to_string() });
    }

    if PART_EXPR.captures(result[1]).is_none() {
        return Err(ParseError::WrongPart { part : result[1].to_string() });
    }

    let action = ActionType::from_str(result[0]).unwrap();
    let part = Part::from_str(result[1]).unwrap();

    Ok(MoveData::new(action, part))
}

pub fn parse_move(params : &str) -> Result<Vec<MoveData>, InputError> {
	let results : Vec<_> = params.split_whitespace()
		.map(part_validator)
		.collect();

	let (ok_iter, err_iter) : (Vec<_>, Vec<_>) = results.into_iter()
		.map(
			|result|
				match result {
					Ok(v) => (Some(v), None),
					Err(e) => (None, Some(e))
				})
		.unzip();

	let errors : Vec<_> = err_iter.into_iter()
		.filter_map(|error| error)
		.collect();

	if !errors.is_empty() {
		return Err(InputError::WrongActions { actions : errors });
	}

	let actions : Vec<_> = ok_iter.into_iter()
		.filter_map(|position| position)
		.collect();

	if actions.len() < 2 {
		return Err(InputError::TooFewActions);
	}

	Ok(actions)
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct BattleToSave {
    pub(crate) warrior_1: Warrior,
    pub(crate) warrior_2: Warrior,
    pub(crate) winner: Option<u32>,
    pub(crate) reward: Balance,
    pub(crate) last_action_timestamp: Timestamp,
    pub(crate) warrior_1_missed_action: bool,
    pub(crate) warrior_2_missed_action: bool,
}

impl From<Battle> for BattleToSave {
    fn from(battle: Battle) -> Self {
        BattleToSave {
            warrior_1: battle.warrior_1.clone(),
            warrior_2: battle.warrior_2.clone(),
            winner: battle.winner,
            reward: 0,
            last_action_timestamp: battle.last_action_timestamp,
            warrior_1_missed_action: battle.warrior_1_missed_action,
            warrior_2_missed_action: battle.warrior_2_missed_action,
        }
    }
}

impl From<BattleToSave> for Battle {
    fn from(battle_to_save: BattleToSave) -> Self {

        let battle = Battle {
            warrior_1: battle_to_save.warrior_1,
            warrior_2: battle_to_save.warrior_2,
            winner: battle_to_save.winner,
            reward: battle_to_save.reward,
            last_action_timestamp: battle_to_save.last_action_timestamp,
            warrior_1_missed_action: battle_to_save.warrior_1_missed_action,
            warrior_2_missed_action: battle_to_save.warrior_2_missed_action,
        };

        battle
    }
}

impl BattleToSave {
    pub fn new(account_id_1: AccountId, account_id_2: AccountId, reward: Option<Balance>) -> BattleToSave {
        let (warrior_1, warrior_2) = BattleToSave::create_two_warriors(account_id_1, account_id_2);

        BattleToSave {
            warrior_1,
            warrior_2,
            winner: None,
            reward: reward.unwrap_or(0),
            last_action_timestamp: env::block_timestamp(),
            warrior_1_missed_action: false,
            warrior_2_missed_action: false,
        }
    }

    fn create_two_warriors(account_id_1: AccountId, account_id_2: AccountId) -> (Warrior, Warrior) {
        // TO DO: Add checking NFT and modify warrior characteristics
        (
            Warrior {
                id: 1,
                account_id: Some(account_id_1),
                strength: BASE_STRENGTH,
                stamina: BASE_STAMINA,
                agility: BASE_AGILITY,
                intuition: BASE_INTUITION,
                health: BASE_HEALTH,
                defense: BASE_DEFENSE,
            },
            Warrior {
                id: 2,
                account_id: Some(account_id_2 + " (bot)"),
                strength: BASE_STRENGTH,
                stamina: BASE_STAMINA,
                agility: BASE_AGILITY,
                intuition: BASE_INTUITION,
                health: BASE_HEALTH,
                defense: BASE_DEFENSE,
            },
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BattleState {
    BattleOver { winner: u32 },
}

impl Battle {
    pub fn apply_actions(&mut self, actions: Vec<MoveData>) -> BattleToSave {
        let parts_map = Part::VARIANTS;
    
        let warrior_1_attack = actions[0].part;
        let warrior_1_protect = actions[1].part;
        
        let attack_seed = *random_seed().get(0).unwrap();
        let protect_seed = *random_seed().get(1).unwrap();
    
        let mut attack_index = attack_seed / 50; 
        let mut protect_index = protect_seed / 50; 
    
        if attack_index > 4 {
            attack_index = attack_index - 4;
        }
        if protect_index > 4 {
            protect_index = protect_index - 4;
        }
    
        let warrior_2_attack = Part::from_str(parts_map[attack_index as usize]).unwrap();
        let warrior_2_protect = Part::from_str(parts_map[protect_index as usize]).unwrap();
    
        let log_message = format!("Attack part: {:?}", parts_map[attack_index as usize]);
        env::log(log_message.as_bytes());
    
        let log_message = format!("Protect part: {:?}", parts_map[protect_index as usize]);
        env::log(log_message.as_bytes());
    
        let log_message = format!("Block timestamp {}", env::block_timestamp());
        env::log(log_message.as_bytes());

        let log_message = format!("Last action timestamp {}", self.last_action_timestamp);
        env::log(log_message.as_bytes());

        if env::block_timestamp() > self.last_action_timestamp + MAX_MS_FOR_ACTION {
            self.warrior_1_missed_action = true;

            let log_message = format!("Time for action is over");
            env::log(log_message.as_bytes());
        } else {
            self.warrior_1_missed_action = false;
        }

        let damage_to_2;
        if self.warrior_1_missed_action {
            damage_to_2 = 0;
        } else {
            if warrior_1_attack != warrior_2_protect || self.warrior_2_missed_action {
                damage_to_2 = 2 * self.warrior_1.strength.clone();
            } else {
                damage_to_2 = 2 * self.warrior_1.strength.clone() - self.warrior_2.defense.clone();
            }
        }
    
        let damage_to_1;
        if self.warrior_2_missed_action {
            damage_to_1 = 0;
        } else {
            if warrior_2_attack != warrior_1_protect || self.warrior_1_missed_action {
                damage_to_1 = 2 * self.warrior_2.strength.clone();
            } else {
                damage_to_1 = 2 * self.warrior_2.strength.clone() - self.warrior_1.defense.clone();
            }
        }
    
        if self.warrior_1.health.clone() > damage_to_1 && self.warrior_2.health.clone() > damage_to_2 {
            let warrior_1_health = self.warrior_1.health.clone() - damage_to_1;
            let warrior_2_health = self.warrior_2.health.clone() - damage_to_2;

            let log_message = format!("Warrior 1 health: {}", warrior_1_health);
            env::log(log_message.as_bytes());

            let log_message = format!("Warrior 2 health: {}", warrior_2_health);
            env::log(log_message.as_bytes());
        
            self.warrior_1.health = warrior_1_health;
            self.warrior_2.health = warrior_2_health;
            self.winner = Some(100);
        
            
            BattleToSave {
                warrior_1: self.warrior_1.clone(),
                warrior_2: self.warrior_2.clone(),
                winner: None,
                reward: 0,
                last_action_timestamp: env::block_timestamp(),
                warrior_1_missed_action: self.warrior_1_missed_action,
                warrior_2_missed_action: self.warrior_2_missed_action,
            }
        } else {
            let mut is_warrior_1_dead = false;
            let mut is_warrior_2_dead = false;
            let mut winner = 0;

            if self.warrior_1.health.clone() <= damage_to_1 {
                is_warrior_1_dead = true;
            }

            if self.warrior_2.health.clone() <= damage_to_2 {
                is_warrior_2_dead = true;
            }

            if is_warrior_1_dead && !is_warrior_2_dead {
                // return Err(BattleState::BattleOver { winner: self.warrior_2.id });
                winner = self.warrior_2.id;
                self.warrior_1.health = 0;
            }

            if !is_warrior_1_dead && is_warrior_2_dead {
                // return Err(BattleState::BattleOver { winner: self.warrior_1.id });
                winner = self.warrior_2.id;
                self.warrior_2.health = 0;
            }

            BattleToSave {
                warrior_1: self.warrior_1.clone(),
                warrior_2: self.warrior_2.clone(),
                winner: Some(winner),
                reward: 0,
                last_action_timestamp: env::block_timestamp(),
                warrior_1_missed_action: self.warrior_1_missed_action,
                warrior_2_missed_action: self.warrior_2_missed_action,
            }
        }
    }

    fn create_two_warriors(account_id_1: AccountId, account_id_2: AccountId) -> (Warrior, Warrior) {
        // TO DO: Add checking NFT and modify warrior characteristics
        (
            Warrior {
                id: 1,
                account_id: Some(account_id_1),
                strength: BASE_STRENGTH,
                stamina: BASE_STAMINA,
                agility: BASE_AGILITY,
                intuition: BASE_INTUITION,
                health: BASE_HEALTH,
                defense: BASE_DEFENSE,
            },
            Warrior {
                id: 2,
                account_id: Some(account_id_2),
                strength: BASE_STRENGTH,
                stamina: BASE_STAMINA,
                agility: BASE_AGILITY,
                intuition: BASE_INTUITION,
                health: BASE_HEALTH,
                defense: BASE_DEFENSE,
            },
        )
    }

    pub fn new(account_id_1: AccountId, account_id_2: AccountId, reward: Option<Balance>) -> Battle {
        let (warrior_1, warrior_2) = Battle::create_two_warriors(account_id_1, account_id_2);

        Battle {
            warrior_1,
            warrior_2,
            winner: None,
            reward: reward.unwrap_or(0),
            last_action_timestamp: env::block_timestamp(),
            warrior_1_missed_action: false,
            warrior_2_missed_action: false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_move(/*params : &str, exp_result : Vec<MoveData>*/) {
        let result = parse_move("Attack:Head Protect:Legs").ok().unwrap();
    
        assert_eq!(vec![MoveData::new(ActionType::Attack, Part::Head), MoveData::new(ActionType::Protect, Part::Legs)], result);
    }
    
    // TO DO: add tests for panics
}