use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Warrior {
	pub id : u32,
    pub(crate) account_id: AccountId,
}
