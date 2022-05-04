use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use crate::ShmapError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub key: String,
    pub expiration: Option<DateTime<Utc>>,
    pub encrypted: bool,
}

impl Metadata {
    pub fn new(key: &str, ttl: Option<std::time::Duration>) -> Result<Self, ShmapError> {
        let expiration = match ttl {
            Some(ttl) => Some(
                Utc::now()
                    + chrono::Duration::from_std(ttl)
                        .map_err(|_| ShmapError::DurationOutOfRangeError)?,
            ),
            None => None,
        };

        Ok(Metadata {
            key: key.to_owned(),
            expiration,
            encrypted: false,
        })
    }
}
