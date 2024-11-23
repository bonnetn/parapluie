use std::collections::Bound;
use tokio::sync::mpsc::Sender;
use crate::error::db::DatabaseError;
use crate::model::item::Item;
use crate::model::partition_key::PartitionKey;
use crate::model::set_value::SetValue;
use crate::model::sort_key::SortKey;

pub enum Task {
    Get {
        partition_key: PartitionKey,
        sort_key: SortKey,
        sender: Sender<Result<Option<Item>, DatabaseError>>,
    },
    Set {
        partition_key: PartitionKey,
        set_value: Vec<SetValue>,
        sender: Sender<Result<bool, DatabaseError>>,
    },
    List {
        partition_key: PartitionKey,
        range: (Bound<SortKey>, Bound<SortKey>),
        page_size: usize,
        sender: Sender<Result<Vec<Item>, DatabaseError>>,
    },
}
