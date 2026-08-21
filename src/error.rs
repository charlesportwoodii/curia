#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("a dispatcher is already installed")]
    AlreadyInstalled,
}
