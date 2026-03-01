#![no_std]
extern crate alloc;

pub mod config;
pub mod error;
#[cfg(feature = "iso")]
mod iso;
pub use error::AppError;
pub mod core;
pub mod path;
pub mod ui;
