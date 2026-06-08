//! Application-wide error types.
//!
//! Centralized error definitions for handling various failure conditions
//! such as UEFI errors, file system errors, and invalid configurations.

#[derive(Debug, thiserror_no_std::Error)]
/// The primary error type for the application.
pub enum AppError {
    #[error(transparent)]
    Uefi(#[from] uefi::Error),
    #[error(transparent)]
    UefiFs(#[from] uefi::fs::Error),
    #[error(transparent)]
    FromStrError(#[from] uefi::data_types::FromStrError),
    #[error(transparent)]
    PathRef(#[from] crate::path::PathRefParseError),
    #[error(transparent)]
    Builder(#[from] uefi::proto::device_path::build::BuildError),
    #[error(transparent)]
    Path(#[from] uefi::proto::device_path::DevicePathUtilitiesError),
    #[error("Error: {0}")]
    Generic(&'static str),
    #[error("NotImplemented: {0}")]
    NotImplemented(&'static str),
}

impl From<uefi::Status> for AppError {
    fn from(status: uefi::Status) -> Self {
        Self::Uefi(uefi::Error::new(status, ()))
    }
}
