use crate::core::app::AppResult;
use crate::core::app::{App, AppCtx, DisplayEntry};
use crate::error::AppError;
use crate::path::{DiskManager, PathReference};
use alloc::borrow::ToOwned as _;
use alloc::string::{String, ToString};
use uefi::CString16;
use uefi::boot::LoadImageSource;
use uefi::cstr16;
use uefi::proto::device_path::PoolDevicePath;
use uefi::proto::loaded_image::LoadedImage;

#[derive(Debug)]
pub enum BootTarget {
    Generic(GenericBootTarget),
    #[cfg(feature = "iso")]
    Iso(crate::iso::IsoBootTarget),
}

/// note: this is a two-way implementation, to allow decisions in the
/// future whether we want to model all targets as enum or use dyn dispatch.
impl BootTarget {
    fn boot(&self, handle: uefi::Handle, dm: &DiskManager) -> Result<(), AppError> {
        match self {
            BootTarget::Generic(target) => target.boot(handle, dm),
            #[cfg(feature = "iso")]
            BootTarget::Iso(target) => target.boot(handle, dm),
        }
    }
}

impl App for BootTarget {
    fn run(&mut self, ctx: &mut AppCtx) -> AppResult {
        match self.boot(ctx.handle, ctx.disk_manager) {
            Ok(_) => AppResult::Booted,
            Err(e) => AppResult::Error(e),
        }
    }
}

impl DisplayEntry for BootTarget {
    fn display_options(&self) -> DisplayOptions {
        match self {
            BootTarget::Generic(target) => target.display_options(),
            #[cfg(feature = "iso")]
            BootTarget::Iso(target) => target.display_options(),
        }
    }
}

pub struct DisplayOptions {
    pub label: String,
}

/// A generic EFI executable + cmd chain-loadable target.
///
/// For example, the Linux kernel can be booted into directly via EFI stub, if compiled with `CONFIG_EFI_STUB=y`, which
/// is a very common default. Windows is also easily chain-loaded.
#[derive(Debug)]
pub struct GenericBootTarget {
    /// Display label for the boot menu
    label: String,
    /// Path to executable. current limitation is that this path is relative
    /// to the root the bootloader is loaded from.
    executable: CString16,
    /// Command options to be passed to LoadedImage::SetLoadOptions
    options: CString16,
}

impl GenericBootTarget {
    pub fn new(
        label: impl AsRef<str>,
        executable: impl AsRef<str>,
        options: impl AsRef<str>,
    ) -> Self {
        Self {
            label: label.as_ref().to_string(),
            executable: CString16::try_from(executable.as_ref())
                .unwrap_or(cstr16!("failed to parse").to_owned()),
            options: CString16::try_from(options.as_ref())
                .unwrap_or(cstr16!("failed to parse").to_owned()),
        }
    }

    fn boot(&self, handle: uefi::Handle, dm: &DiskManager) -> Result<(), AppError> {
        let pathref = PathReference::parse(self.executable.to_string().as_str())?;
        let img_path = dm.resolve_path(&pathref)?;

        log::debug!(
            "Loading image from resolved path: {}",
            path_to_string(&img_path)
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

    fn display_options(&self) -> DisplayOptions {
        DisplayOptions {
            label: self.label.clone(),
        }
    }
}

fn path_to_string(path: &PoolDevicePath) -> CString16 {
    path.to_string(
        uefi::proto::device_path::text::DisplayOnly(true),
        uefi::proto::device_path::text::AllowShortcuts(true),
    )
    .unwrap_or(cstr16!("failed to parse.").into())
}
