# **Shmap**

 **A key-value store based on unix shared-memory files (shm) for persisting state across program restarts.**

 Items are stored in the unix shared memory: it uses `shm_open` to create file in the ramdisk (/dev/shm), then they are mapped in memory with mmap.

 Concurrent access to items it provided thanks to `named-lock` mutexes.

 Value serialization can be made transparently with serde (`bincode`), so don't forget to use [serde_bytes](https://crates.io/crates/serde_bytes) to enable optimized handling of `&[u8]` and `Vec<u8>` !

 Here is a basic example :

 ```rust
 use shmap::{Shmap, ShmapError};
 use std::time::Duration;

 fn main() -> Result<(), ShmapError> {
     let shmap = Shmap::new();

     shmap.insert("key", "value")?;
     let value = shmap.get("key")?;

     assert_eq!(Some("value".to_string()), value);

     Ok(())
 }
 ```

## Todo

- [x] Inter-process Mutex
- [x] Item encryption
- [x] Item TTL
