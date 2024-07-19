use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum OpendalMountError {
    #[snafu(display("Fail to mount fs"))]
    MountError {},

    #[snafu(display("Unsupported scheme type {scheme}"))]
    UnsupportedScheme { scheme: String },

    #[snafu(display("NFSServer FS not found in GraphQL context"))]
    NFSServerNotFound {},
    // #[snafu(display("FS already mounted at {0}"))]
    // AlreadyMounted(String),

    // #[snafu(display("operator creation failure {0}"))]
    // OperatorCreateError(String),

    // #[snafu(display(transparent))]
    // IoError(#[from] std::io::Error),
}

pub type OpendalMountResult<T> = Result<T, OpendalMountError>;
