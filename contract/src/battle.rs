use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Battle {
    warrior_1: Warrior,
    warrior_2: Warrior,
    winner: Warrior,
    reward: Balance
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
