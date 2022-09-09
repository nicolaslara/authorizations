use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageFilterError {
    #[error("Authorization not found")]
    NotFound {},

    #[error("UnauthorizedBecause: {reason:?}")]
    UnauthorizedBecause { reason: String },
}
