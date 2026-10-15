use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuoteError {
    #[error("Invalid ID.")]
    InvalidId,
    #[error("Quote not found.")]
    NotFound,
    #[error("Server error.")]
    ServerError,
    #[error("A quote with this institution / date / key already exists.")]
    Duplicate,
}
