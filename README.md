# shmap

A key-value store based on unix shared-memory files (shm) for persisting state across program restarts.

Values serialization is made with serde (bincode), so don't forget to use [serde_bytes](https://crates.io/crates/serde_bytes) to enable optimized handling of `&[u8]` and `Vec<u8>` !

## Example

```rust
use shmap::{Shmap, ShmapError};

fn main() -> Result<(), ShmapError> {
    let shmap = Shmap::default();

    shmap.set("key", "value")?;
    let value = shmap.get("key")?;
    assert_eq!(Some("value".to_string()), value);

    Ok(())
}
```

## Todo

- [x] Mutex lock
- [ ] Inter-process RwLock
- [ ] Item encryption
- [ ] Item TTL

## Credits

shm module is inspired by [shmemfdrs](https://crates.io/crates/shmemfdrs)
