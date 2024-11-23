use time::OffsetDateTime;
use crate::model::partition_key::PartitionKey;
use crate::model::sort_key::SortKey;

#[derive(Clone, Debug)]
pub struct Item {
    pub partition_key: PartitionKey,
    pub sort_key: SortKey,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub version: u64,
    pub value: Vec<u8>,
}

