//! Implementation for SimpleFileSystemProtocol and FileProtocolV1
//!
//! Utilized by Linux EFI stub. See kernel source at `drivers/firmware/efi/libstub/file.c`.
use alloc::boxed::Box;
use alloc::string::ToString;
use uefi::Guid;
use uefi::Handle;
use uefi_raw::protocol::file_system::FileAttribute;
use uefi_raw::protocol::file_system::FileInfo;
use uefi_raw::protocol::file_system::FileMode;
use uefi_raw::protocol::file_system::FileProtocolRevision;
use uefi_raw::protocol::file_system::FileProtocolV1;
use uefi_raw::protocol::file_system::FileSystemInfo;
use uefi_raw::protocol::file_system::SimpleFileSystemProtocol;
use uefi_raw::Char16;
use uefi_raw::Status;

#[cfg(feature = "iso")]
use crate::iso::iso_file::IsoFile;

#[repr(C)]
struct IsoEfiFs<'a> {
    pub protocol: SimpleFileSystemProtocol,
    buf: &'a iso9660::ISO9660<IsoFile>,
}

impl<'a> IsoEfiFs<'a> {
    unsafe fn new(buf: &'a iso9660::ISO9660<IsoFile>) -> Self {
        Self {
            protocol: SimpleFileSystemProtocol {
                revision: 0,
                open_volume: Self::open_volume,
            },
            buf,
        }
    }
    unsafe extern "efiapi" fn open_volume(
        this: *mut SimpleFileSystemProtocol,
        root: *mut *mut FileProtocolV1,
    ) -> uefi_raw::Status {
        // Cast back to full struct to access state
        let fs = unsafe { &mut *(this as *mut IsoEfiFs) };

        // Allocate a new file handle for root directory
        let entry = iso9660::DirectoryEntry::Directory(fs.buf.root.clone());
        let root_handle = Box::new(IsoFileHandle::new(entry));

        unsafe { *root = Box::into_raw(root_handle) as *mut FileProtocolV1 };
        uefi_raw::Status::SUCCESS
    }
}

#[repr(C)]
struct IsoFileHandle {
    pub protocol: FileProtocolV1,
    entry: iso9660::DirectoryEntry<IsoFile>,
    current_position: usize,
}

impl IsoFileHandle {
    pub fn new(entry: iso9660::DirectoryEntry<IsoFile>) -> Self {
        Self {
            protocol: FileProtocolV1 {
                revision: FileProtocolRevision::REVISION_1,
                open: IsoFileHandle::open,
                close: IsoFileHandle::close,
                read: IsoFileHandle::read,
                delete: IsoFileHandle::delete,
                flush: IsoFileHandle::flush,
                write: IsoFileHandle::write,
                get_info: IsoFileHandle::get_info,
                set_info: IsoFileHandle::set_info,
                get_position: IsoFileHandle::get_position,
                set_position: IsoFileHandle::set_position,
            },
            entry,
            current_position: 0,
        }
    }
    unsafe extern "efiapi" fn open(
        this: *mut FileProtocolV1,
        new_handle: *mut *mut FileProtocolV1,
        file_name: *const Char16,
        _open_mode: FileMode,
        _attributes: FileAttribute,
    ) -> Status {
        let this = unsafe { &*(this as *const IsoFileHandle) };
        let name = unsafe { uefi::CStr16::from_ptr(file_name as *const uefi::Char16) };
        // open only applies to FileProtocolV1 that represents a Directory.
        let file = if let iso9660::DirectoryEntry::Directory(dir) = &this.entry {
            match dir.open(name.to_string().as_str()) {
                Ok(Some(file)) => file,
                Ok(None) => {
                    return Status::NOT_FOUND;
                }
                Err(err) => {
                    log::error!("Failed to open file: {:?}", err);
                    return Status::DEVICE_ERROR;
                }
            }
        } else {
            return Status::INVALID_PARAMETER;
        };
        let proto = Box::new(Self::new(file));
        unsafe { *new_handle = Box::into_raw(proto) as *mut FileProtocolV1 };
        Status::SUCCESS
    }
    unsafe extern "efiapi" fn close(_this: *mut FileProtocolV1) -> Status {
        Status::SUCCESS
    }

    unsafe extern "efiapi" fn delete(_this: *mut FileProtocolV1) -> Status {
        Status::UNSUPPORTED
    }

    unsafe extern "efiapi" fn read(
        this: *mut FileProtocolV1,
        buffer_size: *mut usize,
        buffer: *mut core::ffi::c_void,
    ) -> Status {
        unsafe {
            use iso9660::io::{Read, Seek, SeekFrom};
            let this = &mut *(this as *mut IsoFileHandle);
            let f = match &this.entry {
                iso9660::DirectoryEntry::File(f) => f,
                _ => return Status::UNSUPPORTED,
            };
            let size_to_fill = (*buffer_size).min(f.size() as usize - this.current_position);
            let mut reader = f.read();

            if let Err(e) = reader.seek(SeekFrom::Start(this.current_position as u64)) {
                log::error!("Error seeking in FileProtocolV1 hook: {e}");
                return Status::INVALID_PARAMETER;
            }

            // only write up to size_to_fill so that the read() call doesn't overshoot.
            let buf = core::slice::from_raw_parts_mut(buffer as *mut u8, size_to_fill);
            match reader.read(buf) {
                Ok(read) => {
                    this.current_position += read;
                    *buffer_size = read;
                }
                Err(e) => {
                    log::error!("Failed to read: {:?}", e);
                    return Status::DEVICE_ERROR;
                }
            }
        }
        Status::SUCCESS
    }

    unsafe extern "efiapi" fn write(
        _this: *mut FileProtocolV1,
        _buffer_size: *mut usize,
        _buffer: *const core::ffi::c_void,
    ) -> Status {
        Status::UNSUPPORTED
    }

    unsafe extern "efiapi" fn get_position(
        _this: *const FileProtocolV1,
        _position: *mut u64,
    ) -> Status {
        Status::UNSUPPORTED
    }

    unsafe extern "efiapi" fn set_position(_this: *mut FileProtocolV1, _position: u64) -> Status {
        Status::UNSUPPORTED
    }
    unsafe extern "efiapi" fn get_info(
        this: *mut FileProtocolV1,
        information_type: *const Guid,
        buffer_size: *mut usize,
        buffer: *mut core::ffi::c_void,
    ) -> Status {
        let this = unsafe { &*(this as *const IsoFileHandle) };

        match unsafe { *information_type } {
            uefi_raw::protocol::file_system::FileInfo::ID => {
                fill_file_info(&this.entry, buffer_size, buffer)
            }
            uefi_raw::protocol::file_system::FileSystemInfo::ID => {
                fill_filesystem_info(buffer_size, buffer)
            }
            _ => Status::UNSUPPORTED,
        }
    }

    // Linux kernel does not use this, hence unsupported for now.
    unsafe extern "efiapi" fn set_info(
        _this: *mut FileProtocolV1,
        _information_type: *const Guid,
        _buffer_size: usize,
        _buffer: *const core::ffi::c_void,
    ) -> Status {
        Status::UNSUPPORTED
    }

    // Linux kernel does not use this, hence unsupported for now.
    unsafe extern "efiapi" fn flush(_this: *mut FileProtocolV1) -> Status {
        Status::UNSUPPORTED
    }
}

fn fill_file_info(
    entry: &iso9660::DirectoryEntry<IsoFile>,
    buffer_size: *mut usize,
    buffer: *mut core::ffi::c_void,
) -> Status {
    match entry {
        iso9660::DirectoryEntry::File(f) => fill_file_info_for_file(f, buffer_size, buffer),
        iso9660::DirectoryEntry::Directory(_) => fill_file_info_for_directory(buffer_size, buffer),
    }
}

fn fill_file_info_for_file(
    f: &iso9660::ISOFile<IsoFile>,
    buffer_size: *mut usize,
    buffer: *mut core::ffi::c_void,
) -> Status {
    let filename_chars = f.identifier.encode_utf16().count() + 1;
    let required_size =
        core::mem::size_of::<FileInfo>() + (filename_chars * core::mem::size_of::<Char16>());

    if unsafe { *buffer_size } < required_size || buffer.is_null() {
        unsafe { *buffer_size = required_size };
        return Status::BUFFER_TOO_SMALL;
    }

    unsafe { *buffer_size = required_size };

    let info = buffer as *mut FileInfo;
    unsafe {
        (*info).size = required_size as u64;
        (*info).file_size = f.header.extent_length as u64;
        (*info).physical_size = f.header.extent_length as u64;
        (*info).create_time = core::mem::zeroed();
        (*info).last_access_time = core::mem::zeroed();
        (*info).modification_time = core::mem::zeroed();
        (*info).attribute = FileAttribute::READ_ONLY;

        write_utf16_string(&f.identifier, (*info).file_name.as_mut_ptr());
    }

    Status::SUCCESS
}

fn fill_file_info_for_directory(buffer_size: *mut usize, buffer: *mut core::ffi::c_void) -> Status {
    let required_size = core::mem::size_of::<FileInfo>() + core::mem::size_of::<Char16>();

    if unsafe { *buffer_size } < required_size || buffer.is_null() {
        unsafe { *buffer_size = required_size };
        return Status::BUFFER_TOO_SMALL;
    }

    unsafe { *buffer_size = required_size };

    let info = buffer as *mut FileInfo;
    unsafe {
        (*info).size = required_size as u64;
        (*info).file_size = 0;
        (*info).physical_size = 0;
        (*info).create_time = core::mem::zeroed();
        (*info).last_access_time = core::mem::zeroed();
        (*info).modification_time = core::mem::zeroed();
        (*info).attribute = FileAttribute::DIRECTORY;

        *(*info).file_name.as_mut_ptr() = 0;
    }

    Status::SUCCESS
}

fn fill_filesystem_info(buffer_size: *mut usize, buffer: *mut core::ffi::c_void) -> Status {
    let vol_label = "ISO9660";
    let label_chars = vol_label.len() + 1;
    let required_size =
        core::mem::size_of::<FileSystemInfo>() + (label_chars * core::mem::size_of::<Char16>());

    if unsafe { *buffer_size } < required_size || buffer.is_null() {
        unsafe { *buffer_size = required_size };
        return Status::BUFFER_TOO_SMALL;
    }

    unsafe { *buffer_size = required_size };

    let info = buffer as *mut FileSystemInfo;
    unsafe {
        (*info).size = required_size as u64;
        (*info).read_only = uefi_raw::Boolean::TRUE;
        (*info).volume_size = 0;
        (*info).free_space = 0;
        (*info).block_size = 2048;

        write_utf16_string(vol_label, (*info).volume_label.as_mut_ptr());
    }

    Status::SUCCESS
}

fn write_utf16_string(s: &str, mut ptr: *mut Char16) {
    unsafe {
        for ch in s.encode_utf16() {
            *ptr = Char16::from(ch);
            ptr = ptr.add(1);
        }
        *ptr = 0;
    }
}

pub unsafe fn install_iso_fs(
    handle: Option<uefi::Handle>,
    fs: &iso9660::ISO9660<IsoFile>,
) -> uefi::Result<Handle> {
    let proto = unsafe { IsoEfiFs::new(fs) };
    let boxed_proto = Box::into_raw(Box::new(proto));
    unsafe {
        uefi::boot::install_protocol_interface(
            handle,
            &uefi_raw::protocol::file_system::SimpleFileSystemProtocol::GUID,
            boxed_proto as *mut core::ffi::c_void,
        )
    }
}
