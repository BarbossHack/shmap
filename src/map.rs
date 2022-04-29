use crate::{
    errors::ShmapError,
    shm::{shm_open_read, shm_open_write, shm_unlink},
};
use memmap2::{Mmap, MmapMut};
use named_lock::NamedLock;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha224};
use std::path::Path;

pub struct Shmap {}

impl Shmap {
    pub fn new() -> Self {
        Shmap {}
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>, ShmapError>
    where
        T: DeserializeOwned,
    {
        let key = sanitize_key(key);

        let lock = NamedLock::create(&key)?;
        let _guard = lock.lock()?;

        let mmap = match || -> Result<Mmap, ShmapError> {
            let fd = shm_open_read(&key)?;
            let mmap = unsafe { Mmap::map(fd) }?;
            Ok(mmap)
        }() {
            Ok(mmap) => mmap,
            Err(_) => {
                let _ = self.remove(&key);
                return Ok(None);
            }
        };

        let (value, _): (T, usize) =
            bincode::serde::decode_from_slice(mmap.as_ref(), bincode::config::standard())?;
        Ok(Some(value))
    }

    pub fn set<T>(&self, key: &str, value: T) -> Result<(), ShmapError>
    where
        T: Serialize,
    {
        let key = sanitize_key(key);

        let bytes = bincode::serde::encode_to_vec(&value, bincode::config::standard())?;

        let lock = NamedLock::create(&key)?;
        let _guard = lock.lock()?;

        match || -> Result<(), ShmapError> {
            let fd = shm_open_write(&key, bytes.len())?;
            let mut mmap = unsafe { MmapMut::map_mut(fd) }?;
            mmap.copy_from_slice(bytes.as_slice());
            Ok(())
        }() {
            Ok(_) => {}
            Err(e) => {
                let _ = self.remove(&key);
                return Err(e);
            }
        };

        Ok(())
    }

    pub fn remove(&self, key: &str) -> Result<(), ShmapError> {
        let key = sanitize_key(key);

        let lock = NamedLock::create(&key)?;
        let _guard = lock.lock()?;

        let _ = std::fs::remove_file(Path::new("/tmp").join(format!("{}.lock", &key)));
        shm_unlink(&key)?;
        Ok(())
    }
}

fn sanitize_key(key: &str) -> String {
    let mut hasher = Sha224::new();
    hasher.update(key);
    format!("sham.{:x}", hasher.finalize())
}
