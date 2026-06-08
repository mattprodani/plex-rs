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
//! - Graphical user interface using UEFI GOP (Graphics Output Protocol).

#![deny(clippy::suspicious)]
#![deny(clippy::style)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
#![warn(clippy::todo)]
#![allow(clippy::multiple_crate_versions)]

extern crate alloc;

pub mod config;
pub mod core;
pub mod error;
pub mod helpers;
pub mod path;
pub mod ui;

pub use error::AppError;
