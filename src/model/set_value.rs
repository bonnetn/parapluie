use crate::model::sort_key::SortKey;
use crate::model::write_condition::WriteCondition;

pub struct SetValue {
    pub sort_key: SortKey,
    pub write_condition: WriteCondition,
    pub value: Vec<u8>,
}
