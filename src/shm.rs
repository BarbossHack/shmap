//! Provides functions that create shared memory file descriptors.
//! Inspired by https://github.com/unrelentingtech/shmemfdrs

use crate::errors::ShmapError;

pub fn shm_open_read(name: &str) -> Result<i32, ShmapError> {
    let fd = shm_open(name, libc::O_RDONLY)?;
    if fd < 0 {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::NotFound {
            Err(ShmapError::ShmNotFound)
        } else {
            Err(ShmapError::ShmOpenFailed)
        }
    } else {
        Ok(fd)
    }
}

pub fn shm_open_write(name: &str, length: usize) -> Result<i32, ShmapError> {
    let fd = shm_open(name, libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC)?;
    if fd < 0 {
        return Err(ShmapError::ShmOpenFailed);
    }

    let ret = unsafe { libc::ftruncate(fd, length as libc::off_t) };
    if ret != 0 {
        Err(ShmapError::ShmTruncatFailed(ret))
    } else {
        Ok(fd)
    }
}

fn shm_open(name: &str, flags: i32) -> Result<i32, ShmapError> {
    let name = std::ffi::CString::new(name)?;
    let fd = unsafe { libc::shm_open(name.as_ptr(), flags, 0o600) };
    Ok(fd)
}

pub fn shm_unlink(name: &str) -> Result<(), ShmapError> {
    let name = std::ffi::CString::new(name)?;
    let ret = unsafe { libc::shm_unlink(name.as_ptr()) };

    if ret != 0 {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(ShmapError::ShmUnlinkFailed)
        }
    } else {
        Ok(())
    }
}
