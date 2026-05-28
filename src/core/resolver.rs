//! Resolver pipeline for boot entries.

extern crate alloc;

use alloc::vec::Vec;

use crate::core::bootables::BootTarget;
use crate::error::AppError;
use crate::path::DiskManager;

/// Runtime context provided to resolvers.
pub struct ResolverCtx<'a> {
    /// Disk access helpers for resolution.
    pub disk_manager: Option<&'a DiskManager>,
    /// Image handle for UEFI service calls.
    pub image_handle: uefi::Handle,
}

pub enum Resolver {}

/// Plug-in interface for boot entry discovery.
impl Resolver {
    fn name(&self) -> &'static str {
        todo!();
    }

    fn resolve(&self, _ctx: &ResolverCtx) -> Result<Vec<BootTarget>, AppError> {
        todo!();
    }
}
