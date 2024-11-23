use crate::error::db::DatabaseError;
use crate::error::db::DatabaseError::{FailedToSendRequest, NoRemainingMessageInChannel};
use crate::model::item::Item;
use crate::model::partition_key::PartitionKey;
use crate::model::set_value::SetValue;
use crate::model::sort_key::SortKey;
use crate::model::task::Task;
use std::collections::Bound;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct Repository {
    channel: Sender<Task>,
}


impl Repository {
    pub async fn new(channel: Sender<Task>) -> Repository {
        Repository { channel }
    }

    pub async fn get(&self, partition_key: PartitionKey, sort_key: SortKey) -> Result<Option<Item>, DatabaseError> {
        let (sender, receiver) = mpsc::channel(1);

        let request = Task::Get {
            partition_key,
            sort_key,
            sender,
        };

        self.call(request, receiver).await
    }

    pub async fn set(&self, partition_key: PartitionKey, set_value: Vec<SetValue>) -> Result<bool, DatabaseError> {
        let (sender, receiver) = mpsc::channel(1);

        let request = Task::Set {
            partition_key,
            set_value,
            sender,
        };

        self.call(request, receiver).await
    }

    pub async fn list(&self, partition_key: PartitionKey, range: (Bound<SortKey>, Bound<SortKey>), page_size: usize) -> Result<Vec<Item>, DatabaseError> {
        let (sender, receiver) = mpsc::channel(1);
        let request = Task::List {
            partition_key,
            range,
            page_size,
            sender,
        };

        self.call(request, receiver).await
    }

    async fn call<T>(&self, request: Task, mut receiver: Receiver<Result<T, DatabaseError>>) -> Result<T, DatabaseError> {
        self.channel
            .send(request)
            .await
            .map_err(|e| FailedToSendRequest(Box::new(e)))?;

        receiver.recv()
            .await
            .ok_or(NoRemainingMessageInChannel)?
    }
}

