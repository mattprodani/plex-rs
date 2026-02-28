use alloc::boxed::Box;
use uefi::boot::ScopedProtocol;
use uefi::cstr16;
use uefi::proto::device_path::DevicePath;
use uefi::Identify;

#[repr(C, packed)]
pub struct IsoDevicePath {
    // Vendor node: type(1) + subtype(1) + length(2) + GUID(16) = 20 bytes
    vendor_type: u8,
    vendor_subtype: u8,
    vendor_length: [u8; 2],
    vendor_guid: uefi::Guid,
    // End node: type(1) + subtype(1) + length(2) = 4 bytes
    end_type: u8,
    end_subtype: u8,
    end_length: [u8; 2],
}

impl IsoDevicePath {
    pub fn new() -> Self {
        const ISO_VENDOR_GUID: uefi::Guid = uefi::Guid::from_bytes([
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0,
        ]);

        Self {
            vendor_type: 0x01,      // HARDWARE_DEVICE_PATH
            vendor_subtype: 0x04,   // HW_VENDOR_DP
            vendor_length: [20, 0], // sizeof vendor node in little endian
            vendor_guid: ISO_VENDOR_GUID,
            end_type: 0x7F,     // END_DEVICE_PATH_TYPE
            end_subtype: 0xFF,  // END_ENTIRE_DEVICE_PATH_SUBTYPE
            end_length: [4, 0], // sizeof end node
        }
    }
}

fn install_iso_device_path(handle: uefi::Handle) -> uefi::Result<()> {
    let device_path = IsoDevicePath::new();

    let mut boxed_dp = Box::new(device_path);
    let device_path_ptr = &mut *boxed_dp as *mut _ as *mut core::ffi::c_void;

    unsafe {
        uefi::boot::install_protocol_interface(Some(handle), &DevicePath::GUID, device_path_ptr)?
    };

    Box::leak(boxed_dp);
    Ok(())
}

pub fn install_and_get_iso_device_path(
    handle: uefi::Handle,
) -> uefi::Result<alloc::boxed::Box<ScopedProtocol<DevicePath>>> {
    install_iso_device_path(handle)?;

    // Open the protocol to get the fat pointer
    let proto = unsafe {
        uefi::boot::open_protocol::<uefi::proto::device_path::DevicePath>(
            uefi::boot::OpenProtocolParams {
                handle,
                agent: uefi::boot::image_handle(),
                controller: None,
            },
            uefi::boot::OpenProtocolAttributes::GetProtocol,
        )?
    };

    log::debug!(
        "mock device path: {}",
        proto
            .to_string(
                uefi::proto::device_path::text::DisplayOnly(true),
                uefi::proto::device_path::text::AllowShortcuts(true)
            )
            .unwrap_or(cstr16!("failed to parse.").into()),
    );

    Ok(Box::new(proto))
}
