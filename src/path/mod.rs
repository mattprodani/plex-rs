//! Utilities for reading block devices and locating files
//! specified in config.
use core::str::FromStr;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use log::error;
use uefi::boot::OpenProtocolParams;
use uefi::proto::ProtocolPointer;
use uefi::proto::device_path::{DevicePath, PoolDevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::partition::{GptPartitionEntry, MbrPartitionRecord};
use uefi::{CString16, Handle, Identify};

/// URI-style path reference for locating files across partitions
///
/// Supports two addressing modes:
/// - `boot():/path` - The partition where bootloader was loaded from
/// - `guid:PARTUUID:/path` - Partition identified by GPT PARTUUID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathReference {
    /// Which partition contains the file
    pub location: PartitionReference,
    /// Absolute path within that partition (must start with /)
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionReference {
    /// The partition where the bootloader EFI executable was loaded from
    ///
    /// For UEFI systems: This is determined by examining the LoadedImage
    /// protocol's DeviceHandle, which tells us which partition the firmware
    /// loaded us from.
    ///
    /// Syntax: `boot():`
    /// Example: `boot():/vmlinuz-linux`
    Boot,

    /// Partition identified by GPT Partition GUID (PARTUUID)
    ///
    /// This is the unique identifier from the GPT partition table entry,
    /// NOT the filesystem UUID. Each partition in a GPT table has a unique
    /// GUID assigned when the partition is created.
    ///
    /// To find the PARTUUID on Linux:
    /// ```bash
    /// blkid /dev/nvme0n1p2
    /// # Shows: PARTUUID="550e8400-e29b-41d4-a716-446655440000"
    /// ```
    ///
    /// Or inspect GPT directly:
    /// ```bash
    /// sgdisk -i 2 /dev/nvme0n1
    /// # Shows partition unique GUID
    /// ```
    ///
    /// Syntax: `guid(XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX)/path`
    Guid(uefi::Guid),

    /// Drive is an ISO existing in the partition that the EFI image was loaded
    /// from. Kernel/executable is a path within this ISO filesystem that is provided
    /// to the kernel through the BLOCK_IO and SimpleFileSystem UEFI Protocols.
    ///
    /// Syntax: `iso(/uwuntu.iso)/vmlinuz`
    #[cfg(feature = "iso")]
    Iso(String),
}

impl PathReference {
    /// Parse a URI-style path reference
    ///
    /// # Rules
    /// - Resource and path separated by `:`
    ///
    /// # Examples
    /// ```
    /// PathReference::parse("boot():/vmlinuz-linux")?;
    /// PathReference::parse("boot():/EFI/BOOT/BOOTX64.EFI")?;
    /// PathReference::parse("guid(550e8400-e29b-41d4-a716-446655440000)/vmlinuz")?;
    /// PathReference::parse("iso(myfile.iso)/vmlinuz")?; // with feature 'iso'
    /// ```
    ///
    /// # Errors
    /// - `MissingDelimiter` - No `:` found
    /// - `InvalidPath` - Path doesn't start with `/` or is empty after `:`
    /// - `UnknownResource` - Resource type not recognized
    /// - `InvalidGuid` - GUID format incorrect (wrong length, invalid hex, missing hyphens)
    /// - `InvalidSyntax` - boot() has unexpected content in parens
    pub fn parse(s: &str) -> Result<Self, PathRefParseError> {
        let (resource, path) = s
            .split_once(':')
            .ok_or(PathRefParseError::MissingDelimiter)?;

        let location = PartitionReference::parse(resource)?;

        Ok(PathReference {
            location,
            path: path.to_string(),
        })
    }

    /// Convert back to canonical URI string
    ///
    /// # Example
    /// ```
    /// let uri = path_ref.to_uri();
    /// assert_eq!(uri, "boot():/vmlinuz");
    /// ```
    pub fn to_uri(&self) -> String {
        format!("{}{}", self.location.to_uri_prefix(), self.path)
    }
}

impl PartitionReference {
    /// Parse just the partition reference portion (before the final `:`)
    ///
    /// # Examples
    /// ```
    /// PartitionReference::parse("boot")?;
    /// PartitionReference::parse("guid:550e8400-e29b-41d4-a716-446655440000")?;
    /// ```
    pub fn parse(s: &str) -> Result<Self, PathRefParseError> {
        let Some(lparen) = s.find('(') else {
            return Err(PathRefParseError::InvalidSyntax);
        };

        let scheme = &s[..lparen];
        let arg = s[lparen + 1..]
            .strip_suffix(')')
            .ok_or(PathRefParseError::MissingDelimiter)?;

        match scheme {
            "boot" => Ok(PartitionReference::Boot),
            "guid" => Ok(PartitionReference::Guid(
                uefi::Guid::from_str(arg).map_err(|_| PathRefParseError::InvalidGuid)?,
            )),
            #[cfg(feature = "iso")]
            "iso" => Ok(PartitionReference::Iso(arg.to_string())),
            _ => Err(PathRefParseError::UnknownResource(scheme.to_string())),
        }
    }
    /// Convert to URI prefix (everything before the path)
    ///
    /// # Example
    /// ```
    /// assert_eq!(PartitionReference::Boot.to_uri_prefix(), "boot():");
    /// ```
    pub fn to_uri_prefix(&self) -> String {
        match self {
            PartitionReference::Boot => String::from("boot():"),
            PartitionReference::Guid(guid) => {
                format!("guid({}):", guid)
            }
            #[cfg(feature = "iso")]
            PartitionReference::Iso(iso) => {
                format!("iso({}):", iso)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror_no_std::Error)]
pub enum PathRefParseError {
    /// No `:` separator found between resource and path
    #[error("Missing Delimiter")]
    MissingDelimiter,

    /// Path component doesn't start with `/` or is empty
    #[error("Invalid Path")]
    InvalidPath,

    #[error("Unknown Resource: {0}")]
    /// Unknown resource type (not "boot" or "guid")
    UnknownResource(String),

    /// GUID format invalid
    ///
    /// Valid format: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    /// Must have exactly 36 chars (32 hex + 4 hyphens)
    #[error("Invalid Guid")]
    InvalidGuid,

    /// boot() syntax error (something in the parens)
    #[error("Invalid Syntax")]
    InvalidSyntax,
}

/// Manages partition discovery and path resolution
pub struct DiskManager {
    /// All discovered partitions with their metadata
    partitions: Vec<Partition>,
}

impl DiskManager {
    /// Create a new DiskManager by discovering all partitions
    ///
    /// # Arguments
    /// * `boot_handle` - The handle for the partition containing the bootloader
    ///
    ///     (typically from LoadedImage protocol's device_handle)
    ///
    /// # Process
    /// 1. Call LocateHandleBuffer for BlockIO protocol
    /// 2. Filter to only logical partitions (media.is_logical_partition())
    /// 3. For each partition, extract PARTUUID from device path
    /// 4. Store mapping of PARTUUID -> Handle
    ///
    /// # Errors
    /// Returns error if:
    /// - LocateHandleBuffer fails
    /// - Cannot open BlockIO protocol on any handle
    /// - Cannot allocate memory for partition list
    pub fn new(boot_handle: Handle) -> uefi::Result<Self> {
        use uefi::proto::media::partition::PartitionInfo;
        let mut partitions = Vec::new();

        let boot_device_handle = open_protocol_get::<LoadedImage>(boot_handle)?.device();
        let partition_handles = uefi::boot::locate_handle_buffer(
            uefi::boot::SearchType::ByProtocol(&uefi::proto::media::partition::PartitionInfo::GUID),
        )?;

        for handle in partition_handles.iter() {
            match uefi::boot::open_protocol_exclusive::<PartitionInfo>(*handle) {
                Ok(partition_info) => {
                    partitions.push(Partition {
                        handle: *handle,
                        mbr_partition_info: partition_info.mbr_partition_record().cloned(),
                        gpt_partition_info: partition_info.gpt_partition_entry().cloned(),
                        is_system: partition_info.is_system(),
                        is_boot: boot_device_handle == Some(*handle),
                        #[cfg(feature = "iso")]
                        iso_path: None,
                    });
                }
                Err(e) => {
                    error!("failed to open protocol on a partition: {:?}", e)
                }
            }
        }

        Ok(DiskManager { partitions })
    }
    //
    /// Resolve a partition reference to a UEFI handle
    ///
    /// # Arguments
    /// * `reference` - The partition to locate
    ///
    /// # Returns
    /// The UEFI handle for the partition, suitable for opening SimpleFileSystem
    ///
    /// # Behavior
    /// - Boot: Returns cached boot_handle immediately (O(1))
    /// - Guid: Linear search through partitions for matching GUID (O(n))
    ///
    /// # Errors
    /// - `NOT_FOUND` if GUID doesn't match any discovered partition
    pub fn resolve_path(&self, reference: &PathReference) -> uefi::Result<PoolDevicePath> {
        match self
            .partitions
            .iter()
            .find(|part| reference.location.matches(part))
        {
            Some(partition) => {
                let device_path = open_protocol_get::<DevicePath>(partition.handle)?;
                let mut v = Vec::new();
                let root_to_executable =
                    uefi::proto::device_path::build::DevicePathBuilder::with_vec(&mut v)
                        .push(&uefi::proto::device_path::build::media::FilePath {
                            path_name: &CString16::try_from(reference.path.as_str())
                                .map_err(|_| uefi::Error::new(uefi::Status::NOT_FOUND, ()))?,
                        })
                        .map_err(|_| uefi::Error::new(uefi::Status::NOT_FOUND, ()))?
                        .finalize()
                        .map_err(|_| uefi::Error::new(uefi::Status::NOT_FOUND, ()))?;
                Ok(device_path
                    .append_path(root_to_executable)
                    .map_err(|_| uefi::Error::new(uefi::Status::NOT_FOUND, ()))?)
            }
            None => Err(uefi::Error::new(uefi::Status::NOT_FOUND, ())),
        }
    }
}

/// Metadata about a discovered partition
#[derive(Debug)]
pub struct Partition {
    /// UEFI handle for opening protocols on this partition.
    pub handle: Handle,

    /// GPT pt info, if avail.
    pub gpt_partition_info: Option<GptPartitionEntry>,

    /// MBR pt info, if avail.
    #[allow(dead_code)]
    pub mbr_partition_info: Option<MbrPartitionRecord>,

    /// Whether marked as system partition (not necessarily boot partition).
    #[allow(dead_code)]
    pub is_system: bool,

    pub is_boot: bool,

    #[cfg(feature = "iso")]
    pub iso_path: Option<String>,
}

impl Partition {
    const fn guid(&self) -> Option<uefi::Guid> {
        if let Some(gpt) = self.gpt_partition_info {
            Some(gpt.unique_partition_guid)
        } else {
            None
        }
    }
}

impl PartitionReference {
    fn matches(&self, p: &Partition) -> bool {
        match &self {
            PartitionReference::Boot => p.is_boot,
            PartitionReference::Guid(id) => p.guid().as_ref() == Some(id),
            #[cfg(feature = "iso")]
            PartitionReference::Iso(iso) => p.iso_path.as_ref() == Some(iso),
        }
    }
}

pub fn open_protocol_get<P: ProtocolPointer + ?Sized>(
    handle: Handle,
) -> Result<uefi::boot::ScopedProtocol<P>, uefi::Error> {
    unsafe {
        uefi::boot::open_protocol::<P>(
            OpenProtocolParams {
                handle,
                agent: uefi::boot::image_handle(),
                controller: None,
            },
            uefi::boot::OpenProtocolAttributes::GetProtocol,
        )
    }
}
