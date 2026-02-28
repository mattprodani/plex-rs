use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;

use crate::bootables::{BootTarget, GenericBootTarget};
#[cfg(feature = "iso")]
use crate::iso::IsoBootTarget;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TargetConfig {
    Generic {
        /// Display label for the boot menu
        label: String,
        /// Path to the executable (relative to boot partition root)
        executable: String,
        /// Command line options to pass to the executable
        #[serde(default)]
        options: String,
    },

    #[cfg(feature = "iso")]
    Iso {
        /// Display label for the boot menu
        label: String,

        /// Path within the ISO filesystem to the EFI executable.
        iso_path: String,

        /// Path within the ISO filesystem to the EFI executable.
        /// `None` to search for executable according to the EFI specification rules.
        executable: Option<String>,

        /// Command line options to pass to the executable
        #[serde(default)]
        options: String,
    },
}

impl TargetConfig {
    fn into_boot_target(self) -> BootTarget {
        match self {
            TargetConfig::Generic {
                label,
                executable,
                options,
            } => BootTarget::Generic(GenericBootTarget::new(label, executable, options)),
            #[cfg(feature = "iso")]
            TargetConfig::Iso {
                label,
                iso_path,
                executable,
                options,
            } => BootTarget::Iso(IsoBootTarget {
                label,
                iso_path,
                executable,
                options,
            }),
        }
    }
}

/// Top-level configuration structure
#[derive(Debug, Deserialize)]
pub struct Config {
    /// List of boot targets
    pub boot_targets: Vec<TargetConfig>,
}

impl Config {
    /// Load configuration from a TOML file at the specified path
    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        // Read file from UEFI filesystem
        let contents = read_file_to_string(path)?;

        // Parse TOML
        let config: Config = toml::from_str(&contents).map_err(|e| {
            log::error!("TOML parse error: {:?}", e);
            ConfigError::ParseError
        })?;

        Ok(config)
    }

    /// Convert config into a vector of GenericBootTarget
    pub fn into_boot_targets(self) -> Vec<BootTarget> {
        self.boot_targets
            .into_iter()
            .map(|target| target.into_boot_target())
            .collect()
    }
}

/// Read a file from the UEFI filesystem into a String
fn read_file_to_string(path: &str) -> Result<String, ConfigError> {
    use uefi::fs::FileSystem;
    use uefi::CString16;

    // Convert path to CString16
    let path_cstr = CString16::try_from(path).map_err(|_| ConfigError::InvalidPath)?;

    // Get filesystem protocol
    let mut fs = FileSystem::new(
        uefi::boot::get_image_file_system(uefi::boot::image_handle())
            .map_err(|_| ConfigError::FsError)?,
    );

    // Open and read the file - fs.read() returns Vec<u8> directly
    let buf = fs
        .read(path_cstr.as_ref())
        .map_err(|_| ConfigError::FileNotFound)?;

    // Convert to String (assuming UTF-8)
    String::from_utf8(buf).map_err(|_| ConfigError::EncodingError)
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidPath,
    FileNotFound,
    FsError,
    EncodingError,
    ParseError,
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::InvalidPath => write!(f, "Invalid file path"),
            ConfigError::FileNotFound => write!(f, "Config file not found"),
            ConfigError::FsError => write!(f, "Filesystem error"),
            ConfigError::EncodingError => write!(f, "File encoding error"),
            ConfigError::ParseError => write!(f, "TOML parse error"),
        }
    }
}
