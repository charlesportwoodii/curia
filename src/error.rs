#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("a dispatcher is already installed")]
    AlreadyInstalled,
}

// A sink that owns a file or a clock can fail at construction. Install failure
// stays a separate type because it has one cause and no IO.
#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TimeFormat(#[from] time::error::Format),
    #[error(transparent)]
    InvalidFormatDescription(#[from] time::error::InvalidFormatDescription),
}
