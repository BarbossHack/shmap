use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShmapError {
    #[error("shm_open failed")]
    ShmOpenFailed,

    #[error("shm_truncate failed: {}", _0)]
    ShmTruncatFailed(i32),

    #[error("shm_unlink failed")]
    ShmUnlinkFailed,

    #[error("shm file not found")]
    ShmNotFound,

    #[error("CStringNulError: {}", _0)]
    CStringNulError(#[from] std::ffi::NulError),

    #[error("BincodeDecodeError: {}", _0)]
    BincodeDecodeError(#[from] bincode::error::DecodeError),

    #[error("BincodeEncodeError: {}", _0)]
    BincodeEncodeError(#[from] bincode::error::EncodeError),

    #[error("IO Error: {}", _0)]
    IOError(#[from] std::io::Error),

    #[error("NamedLockError: {}", _0)]
    NamedLockError(#[from] named_lock::Error),

    #[error("DurationOutOfRangeError")]
    DurationOutOfRangeError,
}
