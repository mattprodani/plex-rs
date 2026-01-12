# plex

Plex is a pure Rust GUI UEFI bootloader designed for managing multi-boot systems built for fun and daily personal use.
I spent too much time trying to configure Refind to my liking and thought I'd start working on something else for fun.
I will consider this project "complete" once I have replaced my boot manager with `plex` and begin daily driving it.

Current state: WIP. Working POC of a grub-like selection list with capability to boot into multiple targets. Can boot
any regular EFI application with support for passing commands. e.g. most linux kernels (compiled with EFI stubs), windows, etc.
There is no standalone initramfs support without EFI stubs, but intended. I also have some workflows that require Secure Boot which
I also intend on supporting.

As expected, this is built with `#[no_std]`, and `global_allocator` feature from uefi crate. I don't intend on going alloc-free, as this is intended for PCs rather than small embedded devices.

Build for target {arch}-unknown-uefi

```
  cargo build --target x86_64-unknown-uefi
```
