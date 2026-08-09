use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThinVolume {
    pub id: String,
    pub logical_size: u64,
}

impl ThinVolume {
    pub fn new(id: &str, logical_size: u64) -> Self {
        Self { id: id.to_owned(), logical_size }
    }
}
