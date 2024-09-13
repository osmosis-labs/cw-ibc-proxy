use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub min_disbursal_amount: u64,
    pub channel_id: String,
    pub ibc_timeout: u64,
    pub memo: String,
    pub to_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    DisburseFunds { denom: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetBalancesResponse)]
    GetBalances {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetBalancesResponse {
    pub balances: Vec<Coin>,
}
