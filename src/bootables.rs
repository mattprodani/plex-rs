use crate::errors::BootEntryError;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use uefi::cstr16;
use uefi::{
    CString16,
    boot::{LoadImageSource, ScopedProtocol, open_protocol_exclusive},
    proto::{
        device_path::{DevicePath, PoolDevicePath},
        loaded_image::LoadedImage,
    },
};

/// A trait for a bootable. Should boot the bootable.
pub trait Boot {
    fn boot(&self, handle: uefi::Handle) -> Result<(), BootEntryError>;
}

pub struct DisplayOptions {
    pub label: String,
}

/// A trait for a bootable. Should boot the bootable.
pub trait DisplayBootEntry {
    fn display_options(&self) -> DisplayOptions;
}

impl DisplayBootEntry for GenericBootTarget {
    fn display_options(&self) -> DisplayOptions {
        DisplayOptions {
            label: self.executable.to_string(),
        }
    }
}

/// A generic executable + cmd target. Linux kernel can be booted into
/// directly via EFI stub, if compiled with `CONFIG_EFI_STUB=y`, which
/// is a very common default.
pub struct GenericBootTarget {
    /// Path to executable. current limitation is that this path is relative
    /// to the root the bootloader is loaded from.
    executable: CString16,
    /// Command options to be passed to LoadedImage::SetLoadOptions
    options: CString16,
}

impl GenericBootTarget {
    pub fn new(executable: impl AsRef<str>, options: impl AsRef<str>) -> Self {
        // ok to unwrap i think. would panic if any of the strings provided
        // have inner null char, which i don't think is possible.
        Self {
            executable: CString16::try_from(executable.as_ref()).unwrap(),
            options: CString16::try_from(options.as_ref()).unwrap(),
        }
    }

    fn get_image_path(
        &self,
        root_path: &DevicePath,
    ) -> Result<Box<PoolDevicePath>, BootEntryError> {
        let mut v = Vec::new();
        let root_to_executable =
            uefi::proto::device_path::build::DevicePathBuilder::with_vec(&mut v)
                .push(&uefi::proto::device_path::build::media::FilePath {
                    path_name: &self.executable,
                })?
                .finalize()?;
        Ok(Box::new(root_path.append_path(root_to_executable)?))
    }
}

impl Boot for GenericBootTarget {
    fn boot(&self, handle: uefi::Handle) -> Result<(), BootEntryError> {
        let root_path = get_root_path(handle)?;
        let img_path = self.get_image_path(&root_path)?;
        log::debug!(
            "Root path: {}, img path: {}",
            root_path
                .to_string(
                    uefi::proto::device_path::text::DisplayOnly(true),
                    uefi::proto::device_path::text::AllowShortcuts(true)
                )
                .unwrap_or(cstr16!("failed to parse.").into()),
            img_path
                .to_string(
                    uefi::proto::device_path::text::DisplayOnly(true),
                    uefi::proto::device_path::text::AllowShortcuts(true)
                )
                .unwrap_or(cstr16!("failed to parse.").into()),
        );

        let src = LoadImageSource::FromDevicePath {
            device_path: &img_path,
            boot_policy: Default::default(),
        };

        let loaded_image_handle = uefi::boot::load_image(handle, src)?;
        let mut loaded_img =
            uefi::boot::open_protocol_exclusive::<LoadedImage>(loaded_image_handle)?;
        unsafe {
            loaded_img.set_load_options(
                self.options.as_ptr() as *const u8,
                self.options.num_bytes() as u32,
            );
        }
        Ok(uefi::boot::start_image(loaded_image_handle)?)
    }
}

fn get_root_path(image_handle: uefi::Handle) -> uefi::Result<ScopedProtocol<DevicePath>> {
    let loaded_image = open_protocol_exclusive::<LoadedImage>(image_handle)?;
    // is this a sane unwrap?
    let device_handle = loaded_image.device().expect("to exist");
    open_protocol_exclusive::<DevicePath>(device_handle)
}
