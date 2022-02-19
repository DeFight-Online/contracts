use crate::*;
use near_sdk::ext_contract;
use near_sdk::json_types::{U128};
use near_contract_standards::non_fungible_token::{Token};
use near_sdk::borsh::{self};

#[ext_contract(ext_paras_receiver)]
trait ExternalParasReceiver {
    //Method stored on the enemy fleet that is called in the fire function
    fn nft_tokens_for_owner(
        &mut self,
        account_id: String,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> PromiseOrValue<Vec<u8>>;
    fn nft_get_series_single(
        &mut self,
        token_series_id: TokenSeriesId,
    ) -> PromiseOrValue<TokenSeriesJson>;
}

#[ext_contract(ext_self)]
trait ParasResolver {
    /*
        resolves the promise of the CCC to the enemy fleet as a part of the fire function
    */
    fn resolve_paras_tokens(
        &mut self,
        account_id: String,
        referrer_id: Option<String>,
    ) -> bool;

    fn resolve_paras_token_series(
        &mut self,
    ) -> bool;
}