use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum EStats {
    Current(Stats),
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Stats {
    // TO DO: Need to figure out why I need to add pub keyword to field name
    pub referrer_id: Option<AccountId>,
    pub affiliates: UnorderedSet<AccountId>,
    pub battles_num: u64,
    pub wins_num: u64,
    pub lost_num: u64,
    pub total_reward: UnorderedMap<Option<AccountId>, Balance>,
    pub total_affiliate_reward: UnorderedMap<Option<AccountId>, Balance>,
}

impl From<EStats> for Stats {
    fn from(e_stats: EStats) -> Self {
        match e_stats {
            EStats::Current(stats) => stats,
        }
    }
}

impl Stats {
    pub fn new(account_id: &AccountId) -> Stats {
        Stats {
            referrer_id: None,
            affiliates: UnorderedSet::new(StorageKey::Affiliates { account_id: account_id.clone() }),
            battles_num: 0,
            wins_num: 0,
            lost_num: 0,
            total_reward: UnorderedMap::new(StorageKey::TotalRewards { account_id: account_id.clone() }),
            total_affiliate_reward: UnorderedMap::new(StorageKey::TotalAffiliateRewards { account_id: account_id.clone() }),
        }
    }
}