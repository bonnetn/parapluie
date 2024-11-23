use crate::error::db::DatabaseError;
use tonic::Status;

#[derive(Debug)]
pub enum EndpointError {
    MissingPartitionKey,
    MissingSortKey,
    InvalidPartitionKey,
    InvalidSortKey,
    NotFound,

    DatabaseError(DatabaseError),
}

impl From<EndpointError> for Status {
    fn from(error: EndpointError) -> Self {
        match error {
            EndpointError::MissingPartitionKey => Status::invalid_argument("missing partition key"),
            EndpointError::MissingSortKey => Status::invalid_argument("missing sort key"),
            EndpointError::InvalidPartitionKey => Status::invalid_argument("invalid partition key"),
            EndpointError::InvalidSortKey => Status::invalid_argument("invalid sort key"),
            EndpointError::NotFound => Status::not_found("not found"),
            EndpointError::DatabaseError(e) => Status::internal(e.to_string()),
        }
    }
}

impl std::fmt::Display for EndpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndpointError::MissingPartitionKey => write!(f, "missing partition key"),
            EndpointError::MissingSortKey => write!(f, "missing sort key"),
            EndpointError::InvalidPartitionKey => write!(f, "invalid partition key"),
            EndpointError::InvalidSortKey => write!(f, "invalid sort key"),
            EndpointError::DatabaseError(e) => write!(f, "database error: {}", e),
            EndpointError::NotFound => write!(f, "not found"),
        }
    }
}

impl std::error::Error for EndpointError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EndpointError::MissingPartitionKey => None,
            EndpointError::MissingSortKey => None,
            EndpointError::InvalidPartitionKey => None,
            EndpointError::InvalidSortKey => None,
            EndpointError::DatabaseError(e) => Some(e),
            EndpointError::NotFound => None,
        }
    }
}

impl From<DatabaseError> for EndpointError {
    fn from(error: DatabaseError) -> Self {
        EndpointError::DatabaseError(error)
    }
}


