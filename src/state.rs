use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, IbcTimeout};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub channel_id: String,
    pub ibc_timeout: IbcTimeout,
    pub min_disbursal_amount: u64,
    pub memo: String,
    pub to_address: String,
}

pub const STATE: Item<State> = Item::new("state");