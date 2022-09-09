use std::fmt::{self, Display};

use cosmwasm_std::StdError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct EmptyError {}

// Implement `Display` for `MinMax`.
impl Display for EmptyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

#[derive(Error, Debug)]
pub enum AuthorizationError<ErrorExt = EmptyError> {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("ContractError: {0}")]
    ContractError(ErrorExt),
}
