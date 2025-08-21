use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod entity;
pub mod schema_loader;
pub mod error;

pub use entity::{Entity, EntityDefinition, GenericEntity};
pub use schema_loader::SchemaLoader;
pub use error::{EntitiesError, Result};

// Re-export field types from the fields crate
pub use fields::{Field, FieldType, default_cardinality};

// Re-export commonly used types
pub use entity::default_cacheable;