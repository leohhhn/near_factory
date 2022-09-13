use std::cmp::min;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{Balance, Gas, log, PromiseError};
use near_sdk::{env, ext_contract, json_types::U128, near_bindgen, AccountId, Promise};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata};
use near_sdk::env::promise_batch_action_deploy_contract;
use near_sdk::serde::forward_to_deserialize_any;
use near_sdk::serde_json::json;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct AirbroFactory {}

// 2 NEAR
const MIN_STORAGE: Balance = 2_000_000_000_000_000_000_000_000;
const CALL_GAS: Gas = Gas(300 * 1_000_000_000_0000);

#[near_bindgen]
impl AirbroFactory {
    pub fn create_airdrop(&self,
                          airdrop_name: &String,
                          metadata: &FungibleTokenMetadata,
                          total_supply: &U128,
                          airdrop_admin: &AccountId,
                          //  airdrop_data: &Vec<String>
    ) -> Promise{

        let ft_subaccount_id = AccountId::new_unchecked(
            format!("ft_{}.{}", airdrop_name, env::current_account_id())
        );

        let claimer_subaccount_id = AccountId::new_unchecked(
            format!("claimer_{}.{}", airdrop_name, env::current_account_id())
        );

        // claimer code & args
        const CLAIMER_CODE: &[u8] = include_bytes!("../../../nearcon2022/contract/target/wasm32-unknown-unknown/release/nearconn2022.wasm");
        let claimer_args = json!({ "message": "lol"})
            .to_string().into_bytes().to_vec();

        // deploy claimer
        log!(&claimer_subaccount_id);
        let promise = Promise::new(claimer_subaccount_id.clone())
            .create_account()
            .transfer(MIN_STORAGE*2)
            .deploy_contract(CLAIMER_CODE.to_vec());
        return promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(CALL_GAS)
                .query_greeting_callback(&ft_subaccount_id, metadata, airdrop_admin, total_supply)
        );
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn query_greeting_callback(&self, ft_subacc_id: &AccountId, metadata: &FungibleTokenMetadata, airdrop_admin: &AccountId, total_supply: &U128,
                                   #[callback_result] call_result: Result<(), PromiseError>) -> bool {
        // FT code & args
        const FT_CODE: &[u8] = include_bytes!("../../../ft_owner/contract/target/wasm32-unknown-unknown/release/ft_owner.wasm");
        let ft_args = json!({"total_supply": total_supply, "metadata": metadata, "claimer_account": airdrop_admin})
            .to_string().into_bytes().to_vec();

            // deploy FT
            //log!("{:#?}", ft_args);
        let promise = Promise::new(ft_subacc_id.clone())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(FT_CODE.to_vec())
            .function_call("new".to_string(), ft_args, 0, CALL_GAS);

        let result = true;
        result
    }
}