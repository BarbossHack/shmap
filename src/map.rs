use crate::{
    errors::ShmapError,
    shm::{shm_open_read, shm_open_write, shm_unlink},
};
use memmap2::{Mmap, MmapMut};
use named_lock::NamedLock;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha224};
use std::path::Path;

fn sanitize_key<S>(key: S) -> String
where
    S: AsRef<str>,
{
    let mut hasher = Sha224::new();
    hasher.update(key.as_ref());
    format!("sham.{:x}", hasher.finalize())
}

pub fn get<S, T>(key: S) -> Result<Option<T>, ShmapError>
where
    S: AsRef<str>,
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
            let _ = remove(&key);
            return Ok(None);
        }
    };

    let (value, _): (T, usize) =
        bincode::serde::decode_from_slice(mmap.as_ref(), bincode::config::standard())?;
    Ok(Some(value))
}

pub fn set<S, T>(key: S, value: T) -> Result<(), ShmapError>
where
    S: AsRef<str>,
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
            let _ = remove(&key);
            return Err(e);
        }
    };

    Ok(())
}

pub fn remove<S>(key: S) -> Result<(), ShmapError>
where
    S: AsRef<str>,
{
    let key = sanitize_key(key);

    let lock = NamedLock::create(&key)?;
    let _guard = lock.lock()?;

    let _ = std::fs::remove_file(Path::new("/tmp").join(format!("{}.lock", &key)));
    shm_unlink(&key)?;
    Ok(())
}
