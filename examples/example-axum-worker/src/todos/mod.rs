use ts_rs::TS;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod routes;

/// A single Todo item.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TodoItem {
    pub id: Uuid,
    /// The description of the item.
    pub description: String,
    /// Whether the item was completed.
    pub complete: bool,
}
