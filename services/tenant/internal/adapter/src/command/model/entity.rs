use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Item {
    V1 {
        id: String,
        name: String,
        price: u32,
    },
}

impl From<kernel::Item> for Item {
    fn from(value: kernel::Item) -> Self {
        Self::V1 {
            id: value.id().to_string(),
            name: value.name().to_string(),
            price: value.price(),
        }
    }
}
