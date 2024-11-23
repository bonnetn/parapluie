use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum DatabaseError {
    NoRemainingMessageInChannel,
    FailedToSendRequest(Box<dyn std::error::Error + Send>),
    SqliteError(rusqlite::Error),
}


impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            DatabaseError::NoRemainingMessageInChannel => write!(f, "no remaining message in channel"),
            DatabaseError::FailedToSendRequest(e) => write!(f, "failed to send request: {}", e),
            DatabaseError::SqliteError(e) => write!(f, "sqlite error: {}", e),
        }
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(e: rusqlite::Error) -> Self {
        DatabaseError::SqliteError(e)
    }
}


impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DatabaseError::NoRemainingMessageInChannel => None,
            DatabaseError::FailedToSendRequest(e) => Some(&**e),
            DatabaseError::SqliteError(e) => Some(e),
        }
    }
}
