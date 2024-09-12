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
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Disbursement interval error")]
    DisbursementIntervalError,

    #[error("Insufficient funds")]
    InsufficientFunds,
}
