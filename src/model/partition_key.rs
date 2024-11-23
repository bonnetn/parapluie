use std::hash::Hash;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct PartitionKey(pub String);

pub struct InvalidPartitionKey;

impl TryFrom<&str> for PartitionKey {
    type Error = InvalidPartitionKey;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            // TODO: More conditions.
            Err(InvalidPartitionKey)
        } else {
            Ok(PartitionKey(value.to_string()))
        }
    }
}


impl TryFrom<String> for PartitionKey {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(())
        } else {
            Ok(PartitionKey(value))
        }
    }
}
