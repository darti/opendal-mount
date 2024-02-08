use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpendalMountError {
    #[error("Fail to mount fs")]
    MountError(),

    #[error("Unsupported scheme type {0}")]
    UnsupportedScheme(String),

    #[error("Multiplexed FS not found")]
    MultiplexedNotFound(),

    #[error("FS already mounted at {0}")]
    AlreadyMounted(String),

    #[error("operator creation failure {0}")]
    OperatorCreateError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub type OpendalMountResult<T> = Result<T, OpendalMountError>;
