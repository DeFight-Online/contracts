use crate::*;
use lazy_static::lazy_static;
use strum::{EnumVariantNames, VariantNames};
use regex::Regex;
use std::str::FromStr;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Battle {
    pub warrior_1: Warrior,
    pub warrior_2: Warrior,
    pub winner: Option<usize>,
    pub reward: Balance
}

#[derive(BorshSerialize, BorshDeserialize/*, Serialize, Deserialize*/)]
// #[serde(crate = "near_sdk::serde")]
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

#[derive(PartialEq, EnumVariantNames, Debug)]
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

#[derive(PartialEq)]
pub enum InputError {
    WrongActions { actions : Vec<ParseError> },
    TooFewActions,
}

#[derive(PartialEq)]
pub enum ParseError {
    WrongAction { action : String },
    WrongPart { part : String },
}

#[derive(PartialEq, EnumVariantNames, Debug)]
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

impl ActionType {
    fn as_str(&self) -> &'static str {
        match self {
            ActionType::Attack => "attack",
            ActionType::Protect => "protect"
        }
    }
}

// #[derive(Serialize, Deserialize)]
// #[serde(crate = "near_sdk::serde")]
#[derive(PartialEq, Debug)]
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
    //
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

impl Battle {
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
                life: BASE_LIFE,
                defense: BASE_DEFENSE,
            },
            Warrior {
                id: 2,
                account_id: Some(account_id_2),
                strength: BASE_STRENGTH,
                stamina: BASE_STAMINA,
                agility: BASE_AGILITY,
                intuition: BASE_INTUITION,
                life: BASE_LIFE,
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

    // ptest!(test_parse_move[
        // test_parse_move_1("attack:head protect:legs", vec![MoveData::new("attack", "head"), MoveData::new("protect", "legs")]),
        // test_parse_move_a2_a1("a2 a1", vec![BoardPosition::new(1, 0), BoardPosition::new(0, 0)]),
        // test_parse_move_a1_a2("a1 a2", vec![BoardPosition::new(0, 0), BoardPosition::new(1, 0)]),
        // test_parse_move_a2_a2("a2 a2", vec![BoardPosition::new(1, 0), BoardPosition::new(1, 0)]),
        // test_parse_move_aa1_aa1("aa1 aa1", vec![BoardPosition::new(0, 26), BoardPosition::new(0, 26)]),
        // test_parse_move_aa1_ab1("aa1 ab1", vec![BoardPosition::new(0, 26), BoardPosition::new(0, 27)]),
        // test_parse_move_ab1_aa1("ab1 aa1", vec![BoardPosition::new(0, 27), BoardPosition::new(0, 26)]),
        // test_parse_move_yy99_zz99("yy99 zz99", vec![BoardPosition::new(98, 674), BoardPosition::new(98, 701)]),
        // test_parse_move_aaa99_aaa99("aaa99 aaa99", vec![BoardPosition::new(98, 702), BoardPosition::new(98, 702)]),
        // test_parse_move_xfd13_ahh37("xfd13 ahh37", vec![BoardPosition::new(12, 16383), BoardPosition::new(36, 891)]),
        // test_parse_move_xx123_yy456_zz789("xx123 yy456 zz789", vec![BoardPosition::new(122, 647), BoardPosition::new(455, 674), BoardPosition::new(788, 701)])
    // ]);
}