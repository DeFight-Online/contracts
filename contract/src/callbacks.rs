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
}

#[ext_contract(ext_self)]
trait ParasResolver {
    /*
        resolves the promise of the CCC to the enemy fleet as a part of the fire function
    */
    fn resolve_get(
        &mut self,
    ) -> bool;
}