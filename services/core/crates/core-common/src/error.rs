use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("upstream unavailable: {0}")]
    UpstreamUnavailable(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl CoreError {
    pub fn code(&self) -> i32 {
        match self {
            Self::NotFound(_) => 1,             // CODE_NOT_FOUND
            Self::InvalidArgument(_) => 2,      // CODE_INVALID_ARGUMENT
            Self::UpstreamUnavailable(_) => 6,  // CODE_UPSTREAM_UNAVAILABLE
            Self::Internal(_) => 7,             // CODE_INTERNAL
        }
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
