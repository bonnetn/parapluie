use rusqlite::named_params;
use std::collections::Bound;
use std::ops::Deref;
use time::OffsetDateTime;
use crate::model::item::Item;
use crate::model::partition_key::PartitionKey;
use crate::model::sort_key::SortKey;

const GET_ITEM_STATEMENT: &str = "
    SELECT created_at, updated_at, version, value
    FROM item
    WHERE partition_key = ?1 AND sort_key = ?2";

const SET_ITEM_STATEMENT: &str = "
    WITH previous_row AS (
        SELECT version
        FROM item
        WHERE partition_key = :partition_key AND sort_key = :sort_key
    ),
    can_insert AS (
        SELECT
            CASE
                WHEN NOT EXISTS (SELECT 1 FROM previous_row) AND (
                    :previous_version IS NULL
                    OR :previous_version = 0
                ) THEN 1
                WHEN EXISTS (SELECT 1 FROM previous_row) AND (
                    :previous_version IS NULL
                    OR (SELECT version FROM previous_row) = :previous_version
                ) THEN 1
                ELSE 0
            END AS allowed
    )
    INSERT INTO item (partition_key, sort_key, created_at, updated_at, version, value)
    SELECT
        :partition_key,
        :sort_key,
        :created_at,
        :updated_at,
        COALESCE((SELECT version FROM previous_row), 0) + 1,
        :value
    FROM can_insert
    WHERE allowed = 1
    ON CONFLICT(partition_key, sort_key)
    DO UPDATE SET
        updated_at = excluded.updated_at,
        version = excluded.version,
        value = excluded.value;";

const LIST_QUERY: &str = "
    SELECT sort_key, created_at, updated_at, version, value
    FROM item
    WHERE partition_key = :partition_key
    AND (:gt_sort_key IS NULL OR sort_key > :gt_sort_key)
    AND (:ge_sort_key IS NULL OR sort_key >= :ge_sort_key)
    AND (:lt_sort_key IS NULL OR sort_key < :lt_sort_key)
    AND (:le_sort_key IS NULL OR sort_key <= :le_sort_key)
    ORDER BY partition_key, sort_key ASC
    LIMIT :page_size";


pub struct SQLiteQueryShim<'a, T> {
    conn: &'a T,
}

impl<'a, T> SQLiteQueryShim<'a, T>
where
    T: Deref<Target=rusqlite::Connection>,
{
    pub fn new(conn: &'a T) -> SQLiteQueryShim<T> {
        SQLiteQueryShim { conn }
    }

    pub fn get(&self, partition_key: &PartitionKey, sort_key: &SortKey) -> rusqlite::Result<Option<Item>> {
        let mut stmt = self.conn.prepare(GET_ITEM_STATEMENT)?;

        let mut rows = stmt.query([&partition_key.0, &sort_key.0])?;
        let row = rows.next()?;
        match row {
            Some(row) => {
                let created_at: OffsetDateTime = row.get(0)?;
                let updated_at: OffsetDateTime = row.get(1)?;
                let version: u64 = row.get(2)?;
                let value: Vec<u8> = row.get(3)?;

                Ok(Some(Item {
                    partition_key: partition_key.clone(),
                    sort_key: sort_key.clone(),
                    created_at,
                    updated_at,
                    version,
                    value,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn set(&self, partition_key: PartitionKey, sort_key: SortKey, created_at: OffsetDateTime, updated_at: OffsetDateTime, previous_version: Option<u64>, value: Vec<u8>) -> rusqlite::Result<bool> {
        let mut stmt = self.conn.prepare(SET_ITEM_STATEMENT)?;

        let result = stmt.execute(named_params! {
            ":partition_key": partition_key.0,
            ":sort_key": sort_key.0,
            ":created_at": created_at,
            ":updated_at": updated_at,
            ":previous_version": previous_version,
            ":value": value,
        })?;

        Ok(result == 1)
    }

    pub fn list(&self, partition_key: PartitionKey, range: (Bound<SortKey>, Bound<SortKey>), page_size: usize) -> rusqlite::Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(LIST_QUERY)?;

        let (gt_sort_key, ge_sort_key) = match range.0 {
            Bound::Included(sort_key) => (None, Some(sort_key.0)),
            Bound::Excluded(sort_key) => (Some(sort_key.0), None),
            Bound::Unbounded => (None, None),
        };

        let (lt_sort_key, le_sort_key) = match range.1 {
            Bound::Included(sort_key) => (None, Some(sort_key.0)),
            Bound::Excluded(sort_key) => (Some(sort_key.0), None),
            Bound::Unbounded => (None, None),
        };

        let mut rows = stmt.query_map(
            named_params! {
                ":partition_key": partition_key.0,
                ":gt_sort_key": gt_sort_key,
                ":ge_sort_key": ge_sort_key,
                ":lt_sort_key": lt_sort_key,
                ":le_sort_key": le_sort_key,
                ":page_size": page_size as i64,
            },
            |row| {
                let sort_key: String = row.get(0)?;
                let created_at: OffsetDateTime = row.get(1)?;
                let updated_at: OffsetDateTime = row.get(2)?;
                let version: u64 = row.get(3)?;
                let value: Vec<u8> = row.get(4)?;

                Ok(Item {
                    partition_key: partition_key.clone(),
                    sort_key: SortKey(sort_key),
                    created_at,
                    updated_at,
                    version,
                    value,
                })
            },
        )?;

        let mut items = Vec::with_capacity(page_size);
        while let Some(item) = rows.next() {
            items.push(item?);
        }
        Ok(items)
    }
}

