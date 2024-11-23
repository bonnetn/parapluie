use crate::repository::query_shim::SQLiteQueryShim;
use rusqlite::Connection;
use std::collections::Bound;
use time::OffsetDateTime;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::error::db::{DatabaseError};
use crate::error::app::AppError;
use crate::model::item::Item;
use crate::model::partition_key::PartitionKey;
use crate::model::set_value::SetValue;
use crate::model::sort_key::SortKey;
use crate::model::task::Task;

pub struct Processor {
    conn: Connection,
    receiver: Receiver<Task>,
}

impl Processor {
    pub fn new(conn: Connection, receiver: Receiver<Task>) -> Self {
        Self {
            conn,
            receiver,
        }
    }

    pub fn blocking_process_tasks(mut self) -> Result<(), AppError> {
        while let Some(request) = self.receiver.blocking_recv() {
            self.process_task(request)?;
        }
        // NOTE: The channel is closed when all the senders are dropped, which happens when the gRPC
        // server is stopped.
        println!("Processor task finished");
        Ok(())
    }
    fn process_task(&mut self, request: Task) -> Result<(), AppError> {
        match request {
            Task::Get { partition_key, sort_key, sender } => {
                let result = self.process_get(partition_key, sort_key);
                reply(sender, result)?;
            }
            Task::Set { partition_key, set_value, sender } => {
                let result = self.process_set(partition_key, set_value);
                reply(sender, result)?;
            }
            Task::List { partition_key, range, page_size, sender } => {
                let result = self.process_list(partition_key, range, page_size);
                reply(sender, result)?;
            }
        }
        Ok(())
    }

    fn process_get(&self, partition_key: PartitionKey, sort_key: SortKey) -> Result<Option<Item>, DatabaseError> {
        let conn = &self.conn;
        let store = SQLiteQueryShim::new(&conn);
        let item = store.get(&partition_key, &sort_key)?;
        Ok(item)
    }

    fn process_set(&mut self, partition_key: PartitionKey, set_values: Vec<SetValue>) -> Result<bool, DatabaseError> {
        let txn = (&mut self.conn).transaction()?;
        let store = SQLiteQueryShim::new(&txn);
        let now = OffsetDateTime::now_utc();

        for v in set_values {
            let updated = store.set(partition_key.clone(), v.sort_key, now, now, v.write_condition.version_equals, v.value)?;
            if !updated {
                txn.rollback()?;
                return Ok(false);
            }
        }

        txn.commit()?;
        Ok(true)
    }

    fn process_list(&self, partition_key: PartitionKey, range: (Bound<SortKey>, Bound<SortKey>), page_size: usize) -> Result<Vec<Item>, DatabaseError> {
        let conn = &self.conn;
        let store = SQLiteQueryShim::new(&conn);
        let items = store
            .list(
                partition_key,
                range,
                page_size,
            )?;
        Ok(items)
    }
}
fn reply<T>(sender: Sender<Result<T, DatabaseError>>, result: Result<T, DatabaseError>) -> Result<(), AppError>
where
    T: Send + 'static,
{
    sender.blocking_send(result)?;
    Ok(())
}

