#![no_std]
//! Plex is a pure Rust GUI UEFI bootloader designed for managing multi-boot systems.
//!
//! It reads a TOML configuration file (`\plex.toml`) on the EFI system partition
//! to discover boot targets, presents a graphical boot menu, and executes the
//! selected target.
//!
//! # Features
//! - Pure Rust `no_std` implementation.
//! - TOML-based configuration for defining boot entries.
//! - Support for booting ISO files (with the `iso` feature).
//! - Graphical user interface using UEFI GOP (Graphics Output Protocol).

extern crate alloc;

pub mod config;
pub mod error;
#[cfg(feature = "iso")]
mod iso;
pub use error::AppError;
pub mod core;
pub mod path;
pub mod ui;
