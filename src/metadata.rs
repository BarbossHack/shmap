use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use crate::ShmapError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub expiration: Option<DateTime<Utc>>,
}

impl Metadata {
    pub fn new(ttl: Option<std::time::Duration>) -> Result<Self, ShmapError> {
        let expiration = match ttl {
            Some(ttl) => Some(
                Utc::now()
                    + chrono::Duration::from_std(ttl)
                        .map_err(|_| ShmapError::DurationOutOfRangeError)?,
            ),
            None => None,
        };

        Ok(Metadata { expiration })
    }
}