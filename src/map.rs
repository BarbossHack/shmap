use crate::{
    errors::ShmapError,
    index::Index,
    shm::{shm_open_read, shm_open_write, shm_unlink},
};
use memmap2::{Mmap, MmapMut};
use named_lock::NamedLock;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha224};
use std::{collections::HashMap, path::Path, time::Duration};

const INDEX_KEY: &str = "indexes";

pub struct Shmap {}

impl Default for Shmap {
    fn default() -> Self {
        Self::new()
    }
}

impl Shmap {
    pub fn new() -> Self {
        Shmap {}
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>, ShmapError>
    where
        T: DeserializeOwned,
    {
        let sanitized_key = sanitize_key(key);

        let lock = NamedLock::create(&sanitized_key)?;
        let guard = lock.lock()?;

        let fd = match shm_open_read(&sanitized_key) {
            Ok(fd) => fd,
            Err(e) => match e {
                ShmapError::ShmNotFound => {
                    drop(guard);
                    let _ = self.remove(&sanitized_key);
                    return Ok(None);
                }
                e => return Err(e),
            },
        };
        let mmap = unsafe { Mmap::map(fd) }?;

        let (value, _): (T, usize) =
            bincode::serde::decode_from_slice(mmap.as_ref(), bincode::config::standard())?;
        Ok(Some(value))
    }

    pub fn insert<T>(&self, key: &str, value: T) -> Result<(), ShmapError>
    where
        T: Serialize,
    {
        self._insert(key, value, None)
    }

    pub fn insert_with_ttl<T>(&self, key: &str, value: T, ttl: Duration) -> Result<(), ShmapError>
    where
        T: Serialize,
    {
        self._insert(key, value, Some(ttl))
    }

    fn _insert<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), ShmapError>
    where
        T: Serialize,
    {
        let sanitized_key = sanitize_key(key);

        let bytes = bincode::serde::encode_to_vec(&value, bincode::config::standard())?;

        let lock = NamedLock::create(&sanitized_key)?;
        let guard = lock.lock()?;

        match || -> Result<(), ShmapError> {
            let fd = shm_open_write(&sanitized_key, bytes.len())?;
            let mut mmap = unsafe { MmapMut::map_mut(fd) }?;
            mmap.copy_from_slice(bytes.as_slice());
            Ok(())
        }() {
            Ok(_) => {}
            Err(e) => {
                drop(guard);
                let _ = self.remove(&sanitized_key);
                return Err(e);
            }
        };
        drop(guard);

        if key.ne(INDEX_KEY) {
            self.insert_index(&sanitized_key, Index::new(ttl)?)?;
        }

        Ok(())
    }

    pub fn remove(&self, key: &str) -> Result<(), ShmapError> {
        let sanitized_key = sanitize_key(key);

        let lock = NamedLock::create(&sanitized_key)?;
        let _guard = lock.lock()?;

        if key.ne(INDEX_KEY) {
            self.remove_index(&sanitized_key)?;
        }

        let _ = std::fs::remove_file(Path::new("/tmp").join(format!("{}.lock", &sanitized_key)));
        shm_unlink(&sanitized_key)?;

        Ok(())
    }

    fn get_indexes(&self) -> Result<Option<HashMap<String, Index>>, ShmapError> {
        Ok(self.get(INDEX_KEY)?)
    }

    fn insert_index(&self, key: &str, index: Index) -> Result<(), ShmapError> {
        let lock = NamedLock::create(INDEX_KEY)?;
        let _guard = lock.lock()?;

        let indexes = match self.get_indexes()? {
            Some(indexes) => indexes,
            None => {
                let mut indexes = HashMap::new();
                indexes.insert(key.to_string(), index);
                indexes
            }
        };
        self.set_indexes(indexes)?;
        Ok(())
    }

    fn set_indexes(&self, indexes: HashMap<String, Index>) -> Result<(), ShmapError> {
        self.insert(INDEX_KEY, indexes)?;
        Ok(())
    }

    fn remove_index(&self, key: &str) -> Result<(), ShmapError> {
        let lock = NamedLock::create(INDEX_KEY)?;
        let _guard = lock.lock()?;

        match self.get_indexes()? {
            Some(mut indexes) => {
                indexes.remove(key);
                self.set_indexes(indexes)?;
            }
            None => {}
        }
        Ok(())
    }
}

fn sanitize_key(key: &str) -> String {
    let mut hasher = Sha224::new();
    hasher.update(key);
    format!("sham.{:x}", hasher.finalize())
}
