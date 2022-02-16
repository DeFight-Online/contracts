use crate::*;

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
}