//! Panic handler from uefi-rs.
//! Copied in to avoid test issues as the feature
//! flag `panic_handler` automatically sets the panic handler,
//! which conflicts with std if we want to use the
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::time::Duration;

use uefi::{boot, println};

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    panic_handler_impl(info)
}

fn panic_handler_impl(info: &core::panic::PanicInfo) -> ! {
    println!("[PANIC]: {}", info);

    // Give the user some time to read the message
    if are_boot_services_active() {
        boot::stall(Duration::from_secs(10));
    } else {
        let mut dummy = 0u64;
        // FIXME: May need different counter values in debug & release builds
        for i in 0..300_000_000 {
            unsafe {
                core::ptr::write_volatile(&raw mut dummy, i);
            }
        }
    }

    #[cfg(all(target_arch = "x86_64", feature = "qemu"))]
    {
        // If running in QEMU, use the f4 exit port to signal the error and exit
        use qemu_exit::QEMUExit;
        let custom_exit_success = 3;
        let qemu_exit_handle = qemu_exit::X86::new(0xF4, custom_exit_success);
        qemu_exit_handle.exit_failure();
    }

    #[cfg(any(not(feature = "qemu"), not(target_arch = "x86_64")))]
    {
        // If the system table is available, use UEFI's standard shutdown mechanism
        if let Some(st) = uefi::table::system_table_raw()
            && !unsafe { st.as_ref().runtime_services }.is_null()
        {
            uefi::runtime::reset(
                uefi::runtime::ResetType::SHUTDOWN,
                uefi::Status::ABORTED,
                None,
            );
        }

        // If we don't have any shutdown mechanism handy, the best we can do is loop
        log::error!("Could not shut down, please power off the system manually...");

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                loop {
                    unsafe {
                        // Try to at least keep CPU from running at 100%
                        core::arch::asm!("hlt", options(nomem, nostack));
                    }
                }
            } else if #[cfg(target_arch = "aarch64")] {
                loop {
                    unsafe {
                        // Try to at least keep CPU from running at 100%
                        core::arch::asm!("hlt 420", options(nomem, nostack));
                    }
                }
            } else {
                loop {
                    // just run forever dammit how do you return never anyway
                }
            }
        }
    }
}

/// Return true if boot services are active, false otherwise.
fn are_boot_services_active() -> bool {
    let Some(st) = uefi::table::system_table_raw() else {
        return false;
    };

    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };

    !st.boot_services.is_null()
}
