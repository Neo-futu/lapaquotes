use thiserror::Error;

#[derive(Debug, Error)]
pub enum BiError {
    #[error("bi pool: {0}")]
    Pool(String),
    #[error("bi db: {0}")]
    Db(#[from] cornucopia::tokio_postgres::Error),
    #[error("bi bitable: {0}")]
    Bitable(String),
    #[error("bi config: {0}")]
    Config(String),
    #[error("bi: {0}")]
    Msg(String),
}

impl BiError {
    pub fn bitable(e: impl Into<String>) -> Self {
        Self::Bitable(e.into())
    }
}
