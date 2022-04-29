# shmap

A key-value store based on unix shared-memory files (shm) for persisting state across program restarts.

Values serialization is made with serde (bincode), so don't forget to use [serde_bytes](https://crates.io/crates/serde_bytes) to enable optimized handling of `&[u8]` and `Vec<u8>` !

## Example

```rust
use shmap::Shmap;

fn main() {
    let shmap = Shmap::new();

    shmap.set("key", "value").unwrap();
    let value = shmap.get("key").unwrap();
    assert_eq!(value, Some("value".to_string()));
}
```

## Todo

- [x] Mutex lock
- [ ] Inter-process RwLock
- [ ] Item encryption
- [ ] Item TTL

## Credits

shm module is inspired by [shmemfdrs](https://crates.io/crates/shmemfdrs)
