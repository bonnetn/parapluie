use std::fmt::{Display, Formatter};
use std::net::AddrParseError;
use tokio::sync::mpsc::error::SendError;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(Debug)]
pub enum AppError {
    SqliteError(rusqlite::Error),
    ChannelError(Box<dyn std::error::Error + Send>),
    TonicError(tonic::transport::Error),
    TaskError(tokio::task::JoinError),
    InvalidAddress(AddrParseError),
    TracingSetupError(SetGlobalDefaultError),
    ReflectionServiceSetupError(tonic_reflection::server::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::SqliteError(e) => write!(f, "sqlite error: {}", e),
            AppError::ChannelError(e) => write!(f, "channel error: {}", e),
            AppError::TonicError(e) => write!(f, "tonic error: {}", e),
            AppError::TaskError(e) => write!(f, "task error: {}", e),
            AppError::InvalidAddress(e) => write!(f, "invalid address: {}", e),
            AppError::TracingSetupError(e) => write!(f, "tracing setup error: {}", e),
            AppError::ReflectionServiceSetupError(e) => write!(f, "reflection service setup error: {}", e),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::SqliteError(e) => Some(e),
            AppError::ChannelError(e) => Some(&**e),
            AppError::TonicError(e) => Some(e),
            AppError::TaskError(e) => Some(e),
            AppError::InvalidAddress(e) => Some(e),
            AppError::TracingSetupError(e) => Some(e),
            AppError::ReflectionServiceSetupError(e) => Some(e),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::SqliteError(e)
    }
}

impl<T> From<SendError<T>> for AppError
where
    T: Send + 'static,
{
    fn from(e: SendError<T>) -> Self {
        AppError::ChannelError(Box::new(e))
    }
}

impl From<tonic::transport::Error> for AppError {
    fn from(e: tonic::transport::Error) -> Self {
        AppError::TonicError(e)
    }
}


impl From<tokio::task::JoinError> for AppError {
    fn from(e: tokio::task::JoinError) -> Self {
        AppError::TaskError(e)
    }
}

impl From<AddrParseError> for AppError {
    fn from(e: AddrParseError) -> Self {
        AppError::InvalidAddress(e)
    }
}

impl From<SetGlobalDefaultError> for AppError {
    fn from(e: SetGlobalDefaultError) -> Self {
        AppError::TracingSetupError(e)
    }
}

impl From<tonic_reflection::server::Error> for AppError {
    fn from(e: tonic_reflection::server::Error) -> Self {
        AppError::ReflectionServiceSetupError(e)
    }
}