# shmap

A key-value store based on unix shared-memory files (shm) for persisting state across program restarts.

Values serialization is made with serde (bincode), so don't forget to use [serde_bytes](https://crates.io/crates/serde_bytes) to enable optimized handling of `&[u8]` and `Vec<u8>` !

## Example

```rust
use shmap::{Shmap, ShmapError};
use std::time::Duration;

fn main() -> Result<(), ShmapError> {
    let shmap = Shmap::new()?;

    shmap.insert("key", "value")?;
    let value = shmap.get("key")?;
    assert_eq!(Some("value".to_string()), value);

    shmap.insert_with_ttl("key", "temporary_value", Duration::from_secs(60))?;

    Ok(())
}
```

## Todo

- [x] Inter-process Mutex
- [ ] Inter-process RwLock
- [ ] Item encryption
- [x] Item TTL
