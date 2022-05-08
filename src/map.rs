use crate::{
    errors::ShmapError,
    metadata::Metadata,
    shm::{shm_open_read, shm_open_write, shm_unlink, SHM_DIR},
};
use aes_gcm::{
    aead::{Aead, NewAead},
    Aes256Gcm, Key, Nonce,
};
use chrono::Utc;
use log::warn;
use memmap2::{Mmap, MmapMut};
use named_lock::NamedLock;
use rand::{seq::SliceRandom, thread_rng};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha224};
use std::{path::PathBuf, time::Duration};

const METADATA_SUFFIX: &str = "metadata";
const SHMAP_PREFIX: &str = "shmap";

#[derive(Clone)]
pub struct Shmap {
    cipher: Option<Aes256Gcm>,
}

impl Default for Shmap {
    /// Default is Shmap without encryption.
    fn default() -> Self {
        Self::new()
    }
}

impl Shmap {
    /// Initialize Shmap with no TTL or encryption.
    pub fn new() -> Self {
        Shmap::_new(None)
    }

    /// Initialize Shmap with AES256 encryption key (random bytes).
    pub fn new_with_encryption(encryption_key: &[u8; 32]) -> Self {
        Shmap::_new(Some(encryption_key))
    }

    fn _new(encryption_key: Option<&[u8; 32]>) -> Self {
        fdlimit::raise_fd_limit();

        // If an encryption key was provided, create and store a `cipher` for AES256-GCM
        let cipher = encryption_key.map(|key| {
            let key = Key::from_slice(key);
            Aes256Gcm::new(key)
        });

        let shmap = Shmap { cipher };
        if let Err(e) = shmap.clean() {
            warn!("Error while cleaning shmap keys: {}", e)
        }
        shmap
    }

    /// Get an item value by its key, and deserialize it (using `bincode`) to T.
    pub fn get<T>(&self, key: &str) -> Result<Option<T>, ShmapError>
    where
        T: DeserializeOwned,
    {
        let sanitized_key = sanitize_key(key);

        // Remove item if expired
        let not_found = match self.get_metadata(key)? {
            Some(metadata) => match metadata.expiration {
                Some(expiration) => {
                    let expired = Utc::now().gt(&expiration);
                    if expired {
                        let _ = self.remove(key);
                    }
                    expired
                }
                None => false,
            },
            None => true,
        };
        if not_found {
            return Ok(None);
        }

        self.get_deserialize(&sanitized_key)
    }

    fn get_metadata(&self, key: &str) -> Result<Option<Metadata>, ShmapError> {
        let sanitized_metadata_key = sanitize_metadata_key(key);
        self.get_deserialize(&sanitized_metadata_key)
    }

    fn get_deserialize<T>(&self, sanitized_key: &str) -> Result<Option<T>, ShmapError>
    where
        T: DeserializeOwned,
    {
        match self._get(sanitized_key)? {
            Some(bytes) => {
                let (value, _): (T, usize) =
                    bincode::serde::decode_from_slice(&bytes, bincode::config::standard())?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get an item by its key, without deserialization, as bytes.
    pub fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, ShmapError> {
        let sanitized_key = sanitize_key(key);
        self._get(&sanitized_key)
    }

    fn _get(&self, sanitized_key: &str) -> Result<Option<Vec<u8>>, ShmapError> {
        let lock = NamedLock::with_path(PathBuf::from(SHM_DIR).join(sanitized_key))?;
        let guard = lock.lock()?;

        // Read the item from shm
        let fd = match shm_open_read(sanitized_key) {
            Ok(fd) => fd,
            Err(e) => match e {
                ShmapError::ShmFileNotFound => {
                    // If the shm returns "file not found", return None
                    drop(guard);
                    //let _ = self._remove(sanitized_key); // useless
                    return Ok(None);
                }
                e => return Err(e),
            },
        };
        let mmap = unsafe { Mmap::map(fd) }?;
        if mmap.len() == 0 {
            // If the value is empty, remove it and return None
            drop(guard);
            let _ = self._remove(sanitized_key);
            return Ok(None);
        }

        // If an encryption key was provided, decrypt the value
        let bytes = if let Some(cipher) = &self.cipher {
            let nonce = Nonce::from_slice(&mmap[..12]);
            cipher.decrypt(nonce, &mmap[12..])?
        } else {
            mmap.to_vec()
        };

        Ok(Some(bytes))
    }

    /// Insert a new item, using `bincode` serialization.
    pub fn insert<T>(&self, key: &str, value: T) -> Result<(), ShmapError>
    where
        T: Serialize,
    {
        let sanitized_key = sanitize_key(key);
        self.insert_serialize(&sanitized_key, value)?;
        self.insert_metadata(Metadata::new(key, None, self.cipher.is_some())?)
    }

    /// Insert a new item, using `bincode` serialization, with a TTL.
    pub fn insert_with_ttl<T>(&self, key: &str, value: T, ttl: Duration) -> Result<(), ShmapError>
    where
        T: Serialize,
    {
        let sanitized_key = sanitize_key(key);
        self.insert_serialize(&sanitized_key, value)?;
        self.insert_metadata(Metadata::new(key, Some(ttl), self.cipher.is_some())?)
    }

    /// Insert a new item, without serialization, with a TTL.
    pub fn insert_raw_with_ttl(
        &self,
        key: &str,
        value: &[u8],
        ttl: Duration,
    ) -> Result<(), ShmapError> {
        let sanitized_key = sanitize_key(key);
        self._insert(&sanitized_key, value)?;
        self.insert_metadata(Metadata::new(key, Some(ttl), self.cipher.is_some())?)
    }

    fn insert_metadata(&self, metadata: Metadata) -> Result<(), ShmapError> {
        let sanitize_metadata_key = sanitize_metadata_key(&metadata.key);
        self.insert_serialize(&sanitize_metadata_key, metadata)
    }

    fn insert_serialize<T>(&self, sanitized_key: &str, value: T) -> Result<(), ShmapError>
    where
        T: Serialize,
    {
        let bytes = bincode::serde::encode_to_vec(&value, bincode::config::standard())?;
        self._insert(sanitized_key, &bytes)
    }

    /// Insert a new item, without serialization.
    pub fn insert_raw(&self, key: &str, value: &[u8]) -> Result<(), ShmapError> {
        let sanitized_key = sanitize_key(key);
        self._insert(&sanitized_key, value)
    }

    fn _insert(&self, sanitized_key: &str, value: &[u8]) -> Result<(), ShmapError> {
        // If an encryption key was provided, encrypt the value
        let bytes = if let Some(cipher) = &self.cipher {
            let mut nonce: Vec<u8> = (0..12).collect();
            nonce.shuffle(&mut thread_rng());
            let mut ciphertext = cipher.encrypt(Nonce::from_slice(nonce.as_slice()), value)?;
            nonce.append(&mut ciphertext);
            nonce
        } else {
            value.to_vec()
        };

        let lock = NamedLock::with_path(PathBuf::from(SHM_DIR).join(sanitized_key))?;
        let guard = lock.lock()?;

        // Insert the item to shm
        match || -> Result<(), ShmapError> {
            let fd = shm_open_write(sanitized_key, bytes.len())?;
            let mut mmap = unsafe { MmapMut::map_mut(fd) }?;
            mmap.copy_from_slice(bytes.as_slice());
            Ok(())
        }() {
            Ok(_) => Ok(()),
            Err(e) => {
                drop(guard);
                let _ = self._remove(sanitized_key);
                Err(e)
            }
        }
    }

    /// Remove an item by its key.
    pub fn remove(&self, key: &str) -> Result<(), ShmapError> {
        let sanitized_key = sanitize_key(key);
        self._remove(&sanitized_key)?;
        self.remove_metadata(key)
    }

    fn remove_metadata(&self, key: &str) -> Result<(), ShmapError> {
        let sanitize_metadata_key = sanitize_metadata_key(key);
        self._remove(&sanitize_metadata_key)
    }

    fn _remove(&self, sanitized_key: &str) -> Result<(), ShmapError> {
        let lock = NamedLock::with_path(PathBuf::from(SHM_DIR).join(sanitized_key))?;
        let _guard = lock.lock()?;

        shm_unlink(sanitized_key)?;

        Ok(())
    }

    /// List available keys.
    pub fn keys(&self) -> Result<Vec<String>, ShmapError> {
        self.clean()
    }

    /// Clean expired items.
    pub fn clean(&self) -> Result<Vec<String>, ShmapError> {
        let mut keys = Vec::<String>::new();
        let read_dir = std::fs::read_dir(PathBuf::from(SHM_DIR))?;
        read_dir.into_iter().for_each(|dir_entry_res| {
            if let Ok(dir_entry) = dir_entry_res {
                let filename = dir_entry.file_name().to_string_lossy().to_string();
                if filename.starts_with(SHMAP_PREFIX) && !filename.ends_with(METADATA_SUFFIX) {
                    let metadata_filename =
                        format!("{}.{}", dir_entry.path().to_string_lossy(), METADATA_SUFFIX);
                    match self.get_deserialize::<Metadata>(&metadata_filename) {
                        Ok(Some(metadata)) => match metadata.expiration {
                            Some(expiration) => {
                                if Utc::now().gt(&expiration) {
                                    // Expired, remove item and metadata
                                    let _ = self._remove(&filename);
                                    let _ = self._remove(&metadata_filename);
                                } else {
                                    // Not expired, add to list
                                    keys.push(metadata.key);
                                }
                            }
                            None => {
                                // Not expiration, add to list
                                keys.push(metadata.key);
                            }
                        },
                        Ok(None) => {
                            // Item exists, but metadata not found, remove item
                            let _ = self._remove(&filename);
                        }
                        Err(_) => {
                            // Can't deserialized metadata or something else happens
                        }
                    }
                } else if filename.starts_with(SHMAP_PREFIX) && filename.ends_with(METADATA_SUFFIX)
                {
                    let filename_path = dir_entry.path().to_string_lossy().to_string();
                    let item_filename =
                        filename_path.trim_end_matches(&format!(".{}", METADATA_SUFFIX));
                    if !PathBuf::from(item_filename).exists() {
                        // Metadata exists, but item not found, remove metadata
                        let _ = self._remove(&filename);
                    }
                }
            }
        });
        Ok(keys)
    }
}

fn sanitize_key(key: &str) -> String {
    let mut hasher = Sha224::new();
    hasher.update(key);
    format!("{}.{:x}", SHMAP_PREFIX, hasher.finalize())
}

fn sanitize_metadata_key(key: &str) -> String {
    format!("{}.{}", sanitize_key(key), METADATA_SUFFIX)
}

#[cfg(test)]
mod tests {
    use crate::{tests::map::rand_string, Shmap};

    #[test]
    fn test_metadatas_presence() {
        let shmap = Shmap::new();
        let key = rand_string(10);
        let value = rand_string(50);

        shmap.insert(&key, value).unwrap();
        let _ = shmap.get_metadata(&key).unwrap().unwrap();

        let shmap = Shmap::new();
        shmap.remove(&key).unwrap();
        let should_be_none = shmap.get_metadata(&key).unwrap();
        assert!(should_be_none.is_none());
    }
}
