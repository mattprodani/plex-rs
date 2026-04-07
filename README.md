# plex

ever thought about putting put a `config.toml` file in your EFI partition? I hope not, but if you do, this boot manager is for you.

Plex is a pure Rust GUI UEFI bootloader designed for managing multi-boot systems built for fun and daily personal use.
I spent too much time trying to configure Refind to my liking and thought I'd start working on something else for fun.
I will consider this project "complete" once I have replaced my boot manager with `plex` and begin daily driving it.

Current state: It works on my machine.

## Configuration

Plex loads boot targets from a TOML configuration file located at `\plex.toml` on the EFI system partition.

Example configuration:

```toml

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

## Development

For testing, we need to disable the uefi-rs `panic_handler` feature, as
it conflicts with the `std` one set by the test harness. Use `just test` for
simple testing. We also need to set the target to the one for your system.

```
  cargo test --target x86_64-unknown-linux-gnu --lib --no-default features
```
