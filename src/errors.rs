use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpendalMountError {
    #[error("Fail to mount fs")]
    MountError(),

    #[error("Unsupported scheme type {0}")]
    UnsupportedScheme(String),

    #[error("NFSServer FS not found")]
    NFSServerNotFound(),

    #[error("FS already mounted at {0}")]
    AlreadyMounted(String),

    #[error("operator creation failure {0}")]
    OperatorCreateError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub type OpendalMountResult<T> = Result<T, OpendalMountError>;
