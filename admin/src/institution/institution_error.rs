use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstitutionError {
    #[error("Invalid institution ID.")]
    InvalidId,
    #[error("Institution not found.")]
    NotFound,
    #[error("Server error.")]
    ServerError,
    #[error("Institution create error.")]
    CreateError,
    #[error("An institution with this name already exists.")]
    CreateNameExists,
}
