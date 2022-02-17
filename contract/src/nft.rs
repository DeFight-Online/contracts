use crate::*;
use strum::{EnumVariantNames, VariantNames};
use std::str::FromStr;
use near_sdk::serde::{Deserialize, Serialize};

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

// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub enum EEquipmentConfig {
//     Current(EquipmentConfig)
// }

// impl From<EEquipmentConfig> for EquipmentConfig {
//     fn from(e_equipment_config: EEquipmentConfig) -> Self {
//         match e_equipment_config {
//             EEquipmentConfig::Current(equipment_config) => equipment_config,
//         }
//     }
// }

#[near_bindgen]
impl DeFight {
    pub fn add_token_series(&mut self, ids: String) {
        assert_eq!(self.owner_ids.contains(&env::predecessor_account_id()), true, "ERR_NO_ACCESS");

        for id in ids.split(",") {
            let id = &id.to_owned();

            self.tokens_series.insert(&id);
        }
    }

    pub fn get_token_series(self) -> Vec<TokenId> {
        self.tokens_series.to_vec()
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