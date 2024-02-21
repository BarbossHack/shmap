//! Provides functions that create shared memory file descriptors.
//! Inspired by <https://github.com/unrelentingtech/shmemfdrs>

use crate::errors::ShmapError;
use memmap2::{MmapAsRawDesc, MmapRawDescriptor};
use std::os::unix::io::RawFd;

pub const SHM_DIR: &str = "/dev/shm";

/// File descriptor struct, allowing to close fd on Drop
#[derive(Debug)]
pub struct Fd(RawFd);

impl From<RawFd> for Fd {
    fn from(value: RawFd) -> Self {
        Self(value)
    }
}

impl MmapAsRawDesc for Fd {
    fn as_raw_desc(&self) -> MmapRawDescriptor {
        self.0.as_raw_desc()
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        // SAFETY: libc call is unsafe
        unsafe {
            libc::close(self.0);
        }
    }
}

/// Open shm in readonly.
pub fn open_read(name: &str) -> Result<Fd, ShmapError> {
    let fd = shm_open(name, libc::O_RDONLY)?;
    // On success, returns a file descriptor (a nonnegative integer)
    if fd < 0 {
        let err = std::io::Error::last_os_error();
        // If the error is "file not found", return a custom error, else, errno
        if err.kind() == std::io::ErrorKind::NotFound {
            Err(ShmapError::ShmFileNotFound)
        } else {
            Err(ShmapError::IOError(err))
        }
    } else {
        Ok(fd.into())
    }
}

/// Open shm with read/write rights, and initialze it to `length`size.
pub fn open_write(name: &str, length: usize) -> Result<Fd, ShmapError> {
    let fd = shm_open(name, libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC)?;
    // On success, returns a file descriptor (a nonnegative integer)
    if fd < 0 {
        let err = std::io::Error::last_os_error();
        return Err(ShmapError::IOError(err));
    }

    // SAFETY: libc call is unsafe
    #[allow(clippy::cast_possible_wrap)]
    let ret = unsafe { libc::ftruncate(fd, length as libc::off_t) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        Err(ShmapError::IOError(err))
    } else {
        Ok(fd.into())
    }
}

fn shm_open(name: &str, flags: i32) -> Result<RawFd, ShmapError> {
    let name = std::ffi::CString::new(name)?;
    // SAFETY: libc call is unsafe
    let fd = unsafe { libc::shm_open(name.as_ptr(), flags, 0o600) };
    Ok(fd)
}

/// Unlink (remove) shm by its name.
pub fn unlink(name: &str) -> Result<(), ShmapError> {
    let c_name = std::ffi::CString::new(name)?;
    // SAFETY: libc call is unsafe
    let ret = unsafe { libc::shm_unlink(c_name.as_ptr()) };
    // returns 0 on success, or -1 on error
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        // If the error is "file not found", just ignore, already removed. Else, return errno
        if err.kind() == std::io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(ShmapError::IOError(err))
        }
    } else {
        Ok(())
    }
}
