#[derive(thiserror::Error, Debug)]
pub enum ShmapError {
    #[error("shm file not found")]
    ShmFileNotFound,

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

    #[error("AesGcmError: {}", _0)]
    AesGcmError(#[from] aes_gcm::Error),
}
