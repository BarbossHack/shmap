use crate::{
    errors::ShmapError,
    metadata::Metadata,
    shm::{shm_open_read, shm_open_write, shm_unlink},
};
use chrono::Utc;
use memmap2::{Mmap, MmapMut};
use named_lock::NamedLock;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha224};
use std::{collections::HashMap, path::Path, time::Duration};

const METADATAS_KEY: &str = "shmap_internal_index";

pub struct Shmap {}

impl Shmap {
    pub fn new() -> Result<Self, ShmapError> {
        let shmap = Shmap {};
        shmap.clean()?;
        Ok(shmap)
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
                    let _ = self.remove(&key);
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
                let _ = self.remove(&key);
                return Err(e);
            }
        };
        drop(guard);

        if key.ne(METADATAS_KEY) {
            self.insert_metadata(&key, Metadata::new(ttl)?)?;
        }

        Ok(())
    }

    pub fn remove(&self, key: &str) -> Result<(), ShmapError> {
        let sanitized_key = sanitize_key(key);

        let lock = NamedLock::create(&sanitized_key)?;
        let _guard = lock.lock()?;

        if key.ne(METADATAS_KEY) {
            self.remove_metadata(&key)?;
        }

        let _ = std::fs::remove_file(Path::new("/tmp").join(format!("{}.lock", &sanitized_key)));
        shm_unlink(&sanitized_key)?;

        Ok(())
    }

    fn get_indexes(&self) -> Result<Option<HashMap<String, Index>>, ShmapError> {
        Ok(self.get(INDEX_KEY)?)
    }

    fn insert_metadata(&self, key: &str, index: Metadata) -> Result<(), ShmapError> {
        let lock = NamedLock::create(METADATAS_KEY)?;
        let _guard = lock.lock()?;

        let mut metadatas = match self.get::<HashMap<String, Metadata>>(METADATAS_KEY)? {
            Some(metadatas) => metadatas,
            None => {
                let metadatas = HashMap::new();
                metadatas
            }
        };
        metadatas.insert(key.to_string(), index);
        self.insert(METADATAS_KEY, metadatas)?;
        Ok(())
    }

    fn remove_metadata(&self, key: &str) -> Result<(), ShmapError> {
        let lock = NamedLock::create(METADATAS_KEY)?;
        let _guard = lock.lock()?;

        match self.get::<HashMap<String, Metadata>>(METADATAS_KEY)? {
            Some(mut metadatas) => {
                metadatas.remove(key);
                self.insert(METADATAS_KEY, metadatas)?;
            }
            None => {}
        }
        Ok(())
    }

    /// Clean expired items
    pub fn clean(&self) -> Result<(), ShmapError> {
        if let Some(metadatas) = self.get::<HashMap<String, Metadata>>(METADATAS_KEY)? {
            let lock = NamedLock::create(METADATAS_KEY)?;
            let guard = lock.lock()?;

            let mut items_to_remove: Vec<String> = Vec::new();

            let items_to_keep: HashMap<String, Metadata> = metadatas
                .into_iter()
                .filter(|(key, index)| match index.expiration {
                    Some(expiration) => {
                        let keep = Utc::now().le(&expiration);
                        if !keep {
                            items_to_remove.push(key.to_string());
                        }
                        keep
                    }
                    None => true,
                })
                .collect();
            self.insert(METADATAS_KEY, items_to_keep)?;

            drop(guard);

            items_to_remove.into_iter().for_each(|key| {
                let _ = self.remove(&key);
            });
        }
        Ok(())
    }
}

fn sanitize_key(key: &str) -> String {
    let mut hasher = Sha224::new();
    hasher.update(key);
    format!("sham.{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use crate::{map::METADATAS_KEY, metadata::Metadata, tests::map::rand_string, Shmap};
    use std::collections::HashMap;

    #[test]
    fn test_metadatas_presence() {
        let shmap = Shmap::new().unwrap();
        let key = rand_string(10);
        let value = rand_string(50);

        shmap.insert(&key, value).unwrap();
        let metadatas = shmap
            .get::<HashMap<String, Metadata>>(METADATAS_KEY)
            .unwrap()
            .unwrap();
        assert!(metadatas.contains_key(&key));

        let shmap = Shmap::new().unwrap();
        shmap.remove(&key).unwrap();
        let metadatas = shmap
            .get::<HashMap<String, Metadata>>(METADATAS_KEY)
            .unwrap()
            .unwrap();
        assert!(!metadatas.contains_key(&key));
    }
}
