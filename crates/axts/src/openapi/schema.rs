use serde::{Deserialize, Serialize};

/// A reference to a TypeScript type generated via
/// [`ts-rs`](https://docs.rs/ts-rs).
///
/// This replaces the JSON Schema object that used to be produced when
/// this library generated Open API documents. Instead of embedding a
/// full schema, operations now simply reference the name of the
/// TypeScript type that describes their shape; the actual `.ts`
/// declarations are written to disk with [`crate::generate::export_types`].
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SchemaObject {
    /// The name of the corresponding TypeScript type/interface,
    /// e.g. `"User"` or `"string"` for primitives that are not
    /// backed by a Rust type registered with `ts-rs`.
    pub ts_type: String,
    /// Documentation extracted from the Rust type, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A free-form example value for this schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}

impl SchemaObject {
    /// Create a [`SchemaObject`] that references a fixed, literal
    /// TypeScript type (e.g. a primitive like `"string"`) that is not
    /// backed by a Rust type registered with `ts-rs`.
    pub fn literal(ts_type: impl Into<String>) -> Self {
        Self {
            ts_type: ts_type.into(),
            description: None,
            example: None,
        }
    }

    /// Same as [`SchemaObject::literal`] but with an example value attached.
    pub fn literal_with_example(ts_type: impl Into<String>, example: serde_json::Value) -> Self {
        Self {
            ts_type: ts_type.into(),
            description: None,
            example: Some(example),
        }
    }
}
