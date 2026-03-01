# plex

ever thought about putting put a `config.toml` file in your EFI partition? I hope not, but if you do, this boot manager is for you.

Plex is a pure Rust GUI UEFI bootloader designed for managing multi-boot systems built for fun and daily personal use.
I spent too much time trying to configure Refind to my liking and thought I'd start working on something else for fun.
I will consider this project "complete" once I have replaced my boot manager with `plex` and begin daily driving it.

Current state: It works on my machine, and, I can boot the linux kernel from an ISO, like Ventoy!! For ISO support, ported some iso9660 support to no-std in this fork: [mattprodani/iso9660-no-std-rs](https://github.com/mattprodani/iso9660-no-std-rs)

## Configuration

Plex loads boot targets from a TOML configuration file located at `\plex.toml` on the EFI system partition.

Example configuration:

```toml

# Boot from an ISO file
[[boot_targets]]
type = "iso"
label = "Kali Linux"
iso_path = "my_distro.iso"
executable = "\\EFI\\arch\\vmlinuz-linux.efi"
options = "root=/dev/sda2 rw initrd=\\EFI\\arch\\initramfs-linux.img"

# Example boot target for Arch Linux
[[boot_targets]]
type = "generic"
label = "Arch Linux"
executable = "\\EFI\\arch\\vmlinuz-linux.efi"
options = "root=/dev/sda2 rw initrd=\\EFI\\arch\\initramfs-linux.img"

```

See `plex.toml.example` for more examples.

## Building

Build for target {arch}-unknown-uefi. You'll figure out the rest.

```
  cargo build --target x86_64-unknown-uefi
```
