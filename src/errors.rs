#[derive(Debug, thiserror_no_std::Error)]
pub enum BootEntryError {
    #[error(transparent)]
    Uefi(#[from] uefi::Error),
    #[error(transparent)]
    Builder(#[from] uefi::proto::device_path::build::BuildError),
    #[error(transparent)]
    Path(#[from] uefi::proto::device_path::DevicePathUtilitiesError),
}
