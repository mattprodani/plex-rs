//! Very WIP.

use alloc::{borrow::ToOwned, string::String};
use uefi::{
    CString16,
    boot::ScopedProtocol,
    proto::{loaded_image::LoadedImage, media::fs::SimpleFileSystem},
};

use crate::{bootables::DisplayOptions, errors::AppError};
pub mod iso_device_path;
pub mod iso_file;
pub mod iso_fs;

#[derive(Debug)]
pub struct IsoBootTarget {
    pub label: String,
    /// Path to the ISO, must be in the boot partition.
    pub iso_path: String,
    /// Path to the EFI executable within the ISO filesystem.
    pub executable: Option<String>,
    /// Command options to be passed to LoadedImage::SetLoadOptions
    pub options: String,
}

impl IsoBootTarget {
    pub fn boot(
        &self,
        handle: uefi::Handle,
        _dm: &crate::path::DiskManager,
    ) -> Result<(), AppError> {
        use iso9660::io::Read;
        let fs_proto: ScopedProtocol<SimpleFileSystem> =
            uefi::boot::get_image_file_system(uefi::boot::image_handle())?;
        let mut boot_fs = uefi::fs::FileSystem::new(fs_proto);

        // Convert path to CString16
        let path_cstr: CString16 = CString16::try_from(self.iso_path.as_str())?;

        let data = boot_fs.read(path_cstr.as_ref())?;
        let iso_fs = iso9660::ISO9660::new(iso_file::IsoFile { data, seek: 0 })?;

        let device_path = match IsoBootTarget::install_device(&iso_fs)?.get() {
            Some(device_path) => device_path,
            None => {
                log::error!("Could not find device path protocol that was just installed");
                return Err(AppError::Generic(
                    "Could not find device path protocol that was just installed",
                ));
            }
        }
        .to_owned();

        let executable = self.find_executable()?;

        let executable = match iso_fs.open(executable)? {
            Some(iso9660::DirectoryEntry::File(f)) => f,
            Some(iso9660::DirectoryEntry::Directory(_)) => {
                return Err(AppError::Generic("Executable is a directory"));
            }
            None => {
                return Err(AppError::Generic("Executable is a not found"));
            }
        };

        let size = executable.size();
        let mut buf = alloc::vec![0; size as usize];
        let mut reader = executable.read();
        let bytes_read = reader.read(&mut buf)?;
        log::info!("executable size: {}, bytes read: {}", size, bytes_read);

        log::debug!(
            "MZ signature: {:04x} (expected 5a4d)",
            u16::from_le_bytes([buf[0], buf[1]])
        );

        let src = uefi::boot::LoadImageSource::FromBuffer {
            buffer: buf.as_slice(),
            file_path: Some(&*device_path),
        };

        let loaded_image_handle = uefi::boot::load_image(handle, src)?;
        let mut loaded_img =
            uefi::boot::open_protocol_exclusive::<LoadedImage>(loaded_image_handle)?;
        let options = uefi::CString16::try_from(self.options.as_str())?;
        unsafe {
            loaded_img.set_load_options(options.as_ptr() as *const u8, options.num_bytes() as u32);
        }
        uefi::boot::start_image(loaded_image_handle)?;

        Ok(())
    }
    pub fn display_options(&self) -> crate::bootables::DisplayOptions {
        DisplayOptions {
            label: alloc::format!("ISO loadable {}", self.label),
        }
    }

    fn find_executable(&self) -> Result<&str, AppError> {
        self.executable.as_deref().ok_or(AppError::NotImplemented(
            "Please configure executable path.",
        ))
    }

    /// todo fix, this shouldn't be boxed, just done for borrow checker
    fn install_device(
        fs: &iso9660::ISO9660<iso_file::IsoFile>,
    ) -> uefi::Result<alloc::boxed::Box<ScopedProtocol<uefi::proto::device_path::DevicePath>>> {
        let new_handle = unsafe { iso_fs::install_iso_fs(None, fs)? };
        iso_device_path::install_and_get_iso_device_path(new_handle)
    }
}
