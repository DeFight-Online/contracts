use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum EStats {
    Current(Stats),
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Stats {
    referrer_id: Option<AccountId>,
    affiliates: UnorderedSet<AccountId>,
    games_num: u64,
    wins_num: u64,
    defeats_num: u64,
    total_reward: UnorderedMap<Option<AccountId>, Balance>,
    total_affiliate_reward: UnorderedMap<Option<AccountId>, Balance>,
}

impl From<EStats> for Stats {
    fn from(e_stats: EStats) -> Self {
        match e_stats {
            EStats::Current(stats) => stats,
        }
    }
}