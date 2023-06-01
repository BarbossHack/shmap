//! # **Shmap**
//!
//! **A key-value store based on unix shared-memory files (shm) for persisting state across program restarts.**
//!
//! ## Features
//!
//! - Items are stored in the unix shared memory: it uses `shm_open` to create file in the ramdisk (/dev/shm), then they are mapped in memory with mmap.
//!
//! - Concurrent access to items it provided thanks to `named-lock` mutexes.
//!
//! - Value serialization can be made transparently with serde (`bincode`), so don't forget to use [serde_bytes](https://crates.io/crates/serde_bytes) to enable optimized handling of `&[u8]` and `Vec<u8>` !
//!
//! - You can protect your data with AES256-GCM encryption.
//!
//! - You can add a TTL so that your items won't be available anymore after this duration.
//!
//! ## Example
//!
//! ```rust
//! use shmap::{Shmap, ShmapError};
//! use std::time::Duration;
//!
//! fn main() -> Result<(), ShmapError> {
//!     let shmap = Shmap::new();
//!
//!     shmap.insert("key", "value")?;
//!     let value = shmap.get("key")?;
//!
//!     assert_eq!(Some("value".to_string()), value);
//!
//!     // We strongly advise to use Shmap with TTL to avoid opening too many file descriptors,
//!     // or using too much RAM
//!     shmap.insert_with_ttl("key", "temporary_value", Duration::from_secs(60))?;
//!
//!     shmap.remove("key")?;
//!
//!     Ok(())
//! }
//! ```

mod errors;
mod map;
mod metadata;
mod shm;
#[cfg(test)]
mod tests;

pub use errors::ShmapError;
pub use map::Shmap;
