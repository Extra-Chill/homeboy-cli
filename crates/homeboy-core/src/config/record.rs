use serde::{Deserialize, Serialize};

use crate::Result;

use super::{SetName, SlugIdentifiable};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record<T> {
    pub id: String,
    #[serde(flatten)]
    pub value: T,
}

impl<T: SlugIdentifiable + SetName> Record<T> {
    pub fn new(name: String, mut value: T) -> Result<Self> {
        value.set_name(name);
        let id = value.slug_id()?;
        Ok(Self { id, value })
    }
}
