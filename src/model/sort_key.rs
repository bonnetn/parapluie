#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]

pub struct SortKey(pub String);

pub struct InvalidSortKey;


impl TryFrom<&str> for SortKey {
    type Error = InvalidSortKey;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(InvalidSortKey)
        } else {
            Ok(SortKey(value.to_string()))
        }
    }
}

impl TryFrom<String> for SortKey {
    type Error = InvalidSortKey;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(InvalidSortKey)
        } else {
            Ok(SortKey(value))
        }
    }
}

