use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Warrior {
	pub id : u32,
    pub(crate) account_id: Option<AccountId>,
    pub strength: u16,
    pub stamina: u16,
    pub agility: u16,
    pub intuition: u16,
    pub life: u16,
    pub defense: u16,
}
