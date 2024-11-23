use crate::error::endpoint::EndpointError;
use crate::error::endpoint::EndpointError::{InvalidPartitionKey, InvalidSortKey, MissingPartitionKey, MissingSortKey};
use crate::model::partition_key::PartitionKey;
use crate::model::set_value::SetValue;
use crate::model::sort_key::SortKey;
use crate::model::write_condition::WriteCondition;
use crate::proto::parapluie as proto;
use crate::proto::parapluie::parapluie_db_server::ParapluieDb;
use crate::repository::Repository;
use std::collections::Bound;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::time::SystemTime;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct Service {
    repository: Repository,
}

impl Service {
    pub fn new(repository: Repository) -> Self {
        Self {
            repository
        }
    }
}

#[tonic::async_trait]
impl ParapluieDb for Service {
    async fn set(&self, request: Request<proto::SetRequest>) -> Result<Response<proto::SetResponse>, Status> {
        let request: proto::SetRequest = request.into_inner();

        let partition_key: PartitionKey = convert_partition_key(request.partition_key)?;
        let set_values = request.set_values
            .into_iter()
            .map(|set_value| {
                let sort_key = convert_sort_key(set_value.sort_key)?;

                let version_equals = set_value.write_condition
                    .map(|write_condition| write_condition.version_equals)
                    .unwrap_or(None);

                let value = set_value.value;

                let write_condition = WriteCondition {
                    version_equals,
                };

                let set_value = SetValue {
                    sort_key,
                    write_condition,
                    value,
                };

                Ok(set_value)
            })
            .collect::<Result<Vec<_>, EndpointError>>()?;

        let result = self.repository.set(partition_key, set_values)
            .await
            .map_err(EndpointError::DatabaseError)?;

        Ok(Response::new(proto::SetResponse {
            updated: result,
        }))
    }

    async fn get(&self, request: Request<proto::GetRequest>) -> Result<Response<proto::GetResponse>, Status> {
        let request: proto::GetRequest = request.into_inner();

        let partition_key = convert_partition_key(request.partition_key)?;
        let sort_key = convert_sort_key(request.sort_key)?;

        let result = self.repository.get(partition_key, sort_key)
            .await
            .map_err(EndpointError::DatabaseError)?
            .ok_or(EndpointError::NotFound)?;

        let created_at: SystemTime = result.created_at.into();
        let updated_at: SystemTime = result.updated_at.into();

        let item = proto::Item {
            partition_key: Some(proto::PartitionKey {
                value: result.partition_key.0,
            }),
            sort_key: Some(proto::SortKey {
                value: result.sort_key.0,
            }),
            value: result.value,
            created_at: Some(created_at.into()),
            updated_at: Some(updated_at.into()),
            version: result.version,
        };

        Ok(Response::new(proto::GetResponse {
            item: Some(item),
        }))
    }

    async fn list(&self, request: Request<proto::ListRequest>) -> Result<Response<proto::ListResponse>, Status> {
        let request: proto::ListRequest = request.into_inner();

        let partition_key: PartitionKey = convert_partition_key(request.partition_key)?;

        let range = request.range
            .unwrap_or_default();

        let start = convert_bound(range.start)?;
        let end = convert_bound(range.end)?;

        // TODO: Type for page size.
        let page_size = request.page_size as usize;

        let result = self.repository.list(partition_key, (start, end), page_size)
            .await
            .map_err(EndpointError::DatabaseError)?;

        let items = result.into_iter()
            .map(|item| {
                let created_at: SystemTime = item.created_at.into();
                let updated_at: SystemTime = item.updated_at.into();

                proto::Item {
                    partition_key: Some(proto::PartitionKey {
                        value: item.partition_key.0,
                    }),
                    sort_key: Some(proto::SortKey {
                        value: item.sort_key.0,
                    }),
                    value: item.value,
                    created_at: Some(created_at.into()),
                    updated_at: Some(updated_at.into()),
                    version: item.version,
                }
            })
            .collect();

        Ok(Response::new(proto::ListResponse {
            items,
        }))
    }
}
fn convert_partition_key(key: Option<proto::PartitionKey>) -> Result<PartitionKey, EndpointError> {
    key
        .ok_or(MissingPartitionKey)?
        .value
        .try_into()
        .map_err(|_| InvalidPartitionKey)
}


fn convert_sort_key(key: Option<proto::SortKey>) -> Result<SortKey, EndpointError> {
    key
        .ok_or(MissingSortKey)?
        .value
        .try_into()
        .map_err(|_| InvalidSortKey)
}

fn convert_bound(b: Option<proto::Bound>) -> Result<Bound<SortKey>, EndpointError> {
    let b = b.and_then(|b| b.bound);
    let bound = match b {
        Some(proto::bound::Bound::Included(sort_key)) => {
            let sort_key = convert_sort_key(Some(sort_key))?;
            Included(sort_key)
        }
        Some(proto::bound::Bound::Excluded(sort_key)) => {
            let sort_key = convert_sort_key(Some(sort_key))?;
            Excluded(sort_key)
        }
        Some(proto::bound::Bound::Unbounded(_)) =>
            Unbounded,
        None =>
            Unbounded
    };
    Ok(bound)
}
