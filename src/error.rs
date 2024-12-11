use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Denom not found: {0}")]
    DenomNotFound(String),

    #[error("Insufficient funds")]
    InsufficientFunds,
}
