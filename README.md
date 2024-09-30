# **Shmap**

**A key-value store based on linux shared-memory files (shm) for persisting state across program restarts.**

## Features

- Items are stored in the linux shared memory: it uses `shm_open` to create file in the ramdisk (/dev/shm), then they are mapped in memory with mmap.

- Concurrent access to items it provided thanks to `named-lock` mutexes.

- Value serialization can be made transparently with serde (`bincode`), so don't forget to use [serde_bytes](https://crates.io/crates/serde_bytes) to enable optimized handling of `&[u8]` and `Vec<u8>` !

- You can protect your data with AES256-GCM encryption.

- You can add a TTL so that your items won't be available anymore after this timeout.

## Example

```rust
use shmap::{Shmap, ShmapError};
use std::time::Duration;

fn main() -> Result<(), ShmapError> {
    let shmap = Shmap::new();

    shmap.insert("key", "value")?;
    let value = shmap.get("key")?;

    assert_eq!(Some("value".to_string()), value);

    // It is strongly advised to use TTL to avoid using too much RAM
    shmap.insert_with_ttl("key", "temporary_value", Duration::from_secs(60))?;

    Ok(())
}
```

## Supported OS

Any POSIX linux where `/dev/shm` is mounted. MacOS and any BSD descendants are therefore not supported.

> [man shm_open(3)](https://man7.org/linux/man-pages/man3/shm_open.3.html)

```text
The POSIX shared memory object implementation on Linux makes use of a dedicated tmpfs(5) filesystem that is normally mounted under /dev/shm.
```
