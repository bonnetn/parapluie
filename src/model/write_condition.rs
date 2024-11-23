#[derive(Debug, Clone, Default)]
pub struct WriteCondition {
    pub version_equals: Option<u64>, // 0 means not exists
}
