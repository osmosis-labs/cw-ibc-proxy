use cosmwasm_std::Coin;
use osmosis_std::types::{
    cosmos::bank::v1beta1::{MsgSend, QueryAllBalancesRequest},
    osmosis::poolmanager::v1beta1::SwapAmountInRoute,
};
use osmosis_test_tube::{Account, Bank, Gamm, Module, OsmosisTestApp, Wasm};

use crate::msg::{ExecuteMsg, InstantiateMsg};

#[test]
fn test_swap_exact_amount_in() {
    let app = OsmosisTestApp::new();
    let wasm = Wasm::new(&app);
    let bank = Bank::new(&app);
    let gamm = Gamm::new(&app);

    let accounts = app
        .init_accounts(
            &[
                Coin::new(1000000000000000000u128, "denom1"),
                Coin::new(1000000000000000000u128, "denom2"),
                Coin::new(2000000000000000000u128, "denom3"),
                Coin::new(1000000000000000000u128, "uosmo"),
            ],
            3,
        )
        .unwrap();

    let admin = &accounts[0]; // the one managing the contract
    let coin_feeder = &accounts[1]; // the one who feeds the contract with coins
    let pool_creator = &accounts[2]; // the one who creates the pool

    let pool_id = gamm
        .create_basic_pool(
            &[
                Coin::new(1000000000000000000u128, "denom1").into(),
                Coin::new(1000000000000000000u128, "denom2").into(),
                Coin::new(2000000000000000000u128, "denom3").into(),
            ],
            &pool_creator,
        )
        .unwrap()
        .data
        .pool_id;

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("cw_ibc_proxy.wasm");

    let wasm_byte_code = std::fs::read(path).unwrap();
    let code_id = wasm
        .store_code(&wasm_byte_code, None, &admin)
        .unwrap()
        .data
        .code_id;

    let contract_addr = wasm
        .instantiate(
            code_id,
            &InstantiateMsg {
                channel_id: "channel-0".to_string(),
                ibc_timeout_interval: 1000,
                min_disbursal_amount: 100,
                memo: "memo".to_string(),
                to_address: "osmo1qzskhrcjnk5634mq5s5ssdlw34aj844ke4m343".to_string(),
            },
            None,
            Some("proxy"),
            &[],
            &admin,
        )
        .unwrap()
        .data
        .address;

    bank.send(
        MsgSend {
            from_address: coin_feeder.address(),
            to_address: contract_addr.clone(),
            amount: vec![
                Coin::new(1000000000000000000u128, "denom1").into(),
                Coin::new(1000000000000000000u128, "denom2").into(),
            ],
        },
        &coin_feeder,
    )
    .unwrap();

    let balances = bank
        .query_all_balances(&QueryAllBalancesRequest {
            address: contract_addr.to_string(),
            pagination: None,
            resolve_denom: false,
        })
        .unwrap()
        .balances;

    assert_eq!(
        balances,
        vec![
            Coin::new(1000000000000000000u128, "denom1").into(),
            Coin::new(1000000000000000000u128, "denom2").into(),
        ]
    );

    let res = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::SwapExactAmountIn {
                token_in: Coin::new(1000000000000000000u128, "denom1").into(),
                token_out_min_amount: "100".to_string(),
                routes: vec![SwapAmountInRoute {
                    pool_id,
                    token_out_denom: "denom3".to_string(),
                }],
            },
            &[],
            admin,
        )
        .unwrap();

    let token_out: Coin = res
        .events
        .iter()
        .find(|event| event.ty == "token_swapped")
        .unwrap()
        .attributes
        .iter()
        .find(|attr| attr.key == "tokens_out")
        .unwrap()
        .value
        .clone()
        .parse::<Coin>()
        .unwrap();

    let balances = bank
        .query_all_balances(&QueryAllBalancesRequest {
            address: contract_addr.to_string(),
            pagination: None,
            resolve_denom: false,
        })
        .unwrap()
        .balances;

    assert_eq!(
        balances,
        vec![
            Coin::new(1000000000000000000u128, "denom2").into(),
            token_out.into()
        ]
    );
}
