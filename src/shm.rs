//! Provides functions that create shared memory file descriptors.
//! Inspired by https://github.com/unrelentingtech/shmemfdrs

use crate::errors::ShmapError;

pub fn shm_open_read<T: AsRef<str>>(name: T) -> Result<i32, ShmapError> {
    shm_open(name, libc::O_RDONLY)
}

pub fn shm_open_write<T: AsRef<str>>(name: T, length: usize) -> Result<i32, ShmapError> {
    let fd = shm_open(name, libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC)?;
    let ret = unsafe { libc::ftruncate(fd, length as libc::off_t) };
    if ret != 0 {
        Err(ShmapError::ShmTruncatFailed(ret))
    } else {
        Ok(fd)
    }
}

fn shm_open<T: AsRef<str>>(name: T, flags: i32) -> Result<i32, ShmapError> {
    let name = std::ffi::CString::new(name.as_ref())?;
    let fd = unsafe { libc::shm_open(name.as_ref().as_ptr(), flags, 0o600) };
    if fd < 0 {
        Err(ShmapError::ShmOpenFailed(fd))
    } else {
        Ok(fd)
    }
}

pub fn shm_unlink<T: AsRef<str>>(name: T) -> Result<(), ShmapError> {
    let name = std::ffi::CString::new(name.as_ref())?;
    let ret = unsafe { libc::shm_unlink(name.as_ref().as_ptr()) };
    if ret != 0 {
        Err(ShmapError::ShmUnlinkFailed(ret))
    } else {
        Ok(())
    }
}
