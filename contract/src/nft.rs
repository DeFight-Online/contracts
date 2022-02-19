use crate::*;
use strum::{EnumVariantNames, VariantNames};
use std::str::FromStr;
use near_sdk::serde::{Deserialize, Serialize};

pub type TokenSeriesId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesJson {
    pub token_series_id: TokenSeriesId,
	pub metadata: TokenMetadata,
	pub creator_id: AccountId,
    pub royalty: HashMap<AccountId, u32>
}

#[derive(PartialEq, EnumVariantNames, Debug, Copy, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum Place {
    Helmet,
    Armor,
    Gloves,
    Bracers,
    ShoulderPads,
    Leggings,
    Boots,
    Amulet,
    Weapon1,
    Weapon2
}

impl FromStr for Place {
    type Err = ();

    fn from_str(input: &str) -> Result<Place, Self::Err> {
        match input {
            "Helmet" => Ok(Place::Helmet),
            "Armor" => Ok(Place::Armor),
            "Gloves" => Ok(Place::Gloves),
            "Bracers" => Ok(Place::Bracers),
            "ShoulderPads" => Ok(Place::ShoulderPads),
            "Leggings" => Ok(Place::Leggings),
            "Boots" => Ok(Place::Boots),
            "Amulet" => Ok(Place::Amulet),
            "Weapon1" => Ok(Place::Weapon1),
            "Weapon2" => Ok(Place::Weapon2),
            _ => Err(()),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EquipmentConfig {
    pub(crate) helmet: Option<TokenId>,
    pub(crate) armor: Option<TokenId>,
    pub(crate) gloves: Option<TokenId>,
    pub(crate) bracers: Option<TokenId>,
    pub(crate) shoulder_pads: Option<TokenId>,
    pub(crate) leggings: Option<TokenId>,
    pub(crate) boots: Option<TokenId>,
    pub(crate) amulet: Option<TokenId>,
    pub(crate) weapon_1: Option<TokenId>,
    pub(crate) weapon_2: Option<TokenId>,
}

#[near_bindgen]
impl DeFight {
    // pub fn is_token_equipped(self, equipment: &EquipmentConfig, place: &str, token_id: TokenId) -> bool {
    //     match place {
    //         "helmet" => equipment.helmet == Some(token_id),
    //         "armor" => equipment.armor == Some(token_id),
    //         "gloves" => equipment.gloves == Some(token_id),
    //         "bracers" => equipment.bracers == Some(token_id),
    //         "shoulder_pads" => equipment.shoulder_pads == Some(token_id),
    //         "leggings" => equipment.leggings == Some(token_id),
    //         "boots" => equipment.boots == Some(token_id),
    //         "amulet" => equipment.amulet == Some(token_id),
    //         "weapon_1" => equipment.weapon_1 == Some(token_id),
    //         "weapon_2" => equipment.weapon_2 == Some(token_id),
    //         _ => false
    //     }
    // }

    pub fn resolve_paras_token_series(&mut self) {
        let log_message = format!("Get token series cross-contract callback");
        env::log(log_message.as_bytes());

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic(b"Unable to get user tokens"),
            PromiseResult::Successful(result) => {
                let token_series = near_sdk::serde_json::from_slice::<TokenSeriesJson>(&result).unwrap();

                let log_message = format!("User tokens: {:?}", token_series);
                env::log(log_message.as_bytes());

                self.tokens_series.insert(&token_series.token_series_id, &token_series);
            },
        }
    }

    pub fn add_token_series(&mut self, id: String) {
        assert_eq!(self.owner_ids.contains(&env::predecessor_account_id()), true, "ERR_NO_ACCESS");
        // let id = &id.to_owned();
        let log_message = format!("Token series id: {:?}", id);
        env::log(log_message.as_bytes());

        ext_paras_receiver::nft_get_series_single(
            id,
            &"paras-token-v2.testnet".to_string(), //contract account to make the call to
            0, //attached deposit
            30_000_000_000_000,
        )

        //we then resolve the promise and call nft_resolve_transfer on our own contract
        .then(ext_self::resolve_paras_token_series(
            &env::current_account_id(), //contract account to make the call to
            0, //attached deposit
            30_000_000_000_000, //GAS attached to the call
        ));
        
    }

    pub fn get_token_series(self, from_index: u64, limit: u64) -> Vec<(TokenId, TokenSeriesJson)> {
        let keys = self.tokens_series.keys_as_vector();
        let values = self.tokens_series.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| {
                let token_series_json: TokenSeriesJson = values.get(index).unwrap().into();
                (keys.get(index).unwrap(), token_series_json.into())
            })
            .collect()
    }

    pub fn get_warrior_equipment(self, account_id: AccountId) -> EquipmentConfig {
        if let Some(equipment) = self.warriors_equipment.get(&account_id) {
            equipment
        } else {
            EquipmentConfig {
                helmet: None,
                armor: None,
                gloves: None,
                bracers: None,
                shoulder_pads: None,
                leggings: None,
                boots: None,
                amulet: None,
                weapon_1: None,
                weapon_2: None,
            }
        }
    }

    pub fn change_warrior_equipment(&mut self, equipment: EquipmentConfig) {
        let account_id = &env::predecessor_account_id();

        // TO DO Data verification for each field of stricture
        self.warriors_equipment.insert(account_id, &equipment);
    }
}