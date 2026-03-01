alias b := build

all: build cp run

test:
  cargo test --target x86_64-unknown-linux-gnu --no-default-features --lib

build:
  cargo build --target x86_64-unknown-uefi

run:
  qemu-system-x86_64 -enable-kvm \
    -drive if=pflash,format=raw,readonly=on,file=.qemu/OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=.qemu/OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:.qemu/esp \
    -serial stdio \
    -m 1G

retrieve-ovmf:
  mkdir -p ./.qemu
  curl -o .qemu/OVMF_CODE.fd https://raw.githubusercontent.com/retrage/edk2-nightly/ebb83e5475d49418afc32857f66111949928bcdc/bin/RELEASEX64_OVMF_CODE.fd
  curl -o .qemu/OVMF_VARS.fd https://raw.githubusercontent.com/retrage/edk2-nightly/ebb83e5475d49418afc32857f66111949928bcdc/bin/RELEASEX64_OVMF_VARS.fd


qemu-setup:
  mkdir -p ./.qemu
  @just cp

cp:
  mkdir -p ./.qemu/esp/efi/boot
  cp ./target/x86_64-unknown-uefi/debug/plex-boot.efi .qemu/esp/efi/boot/bootx64.efi
