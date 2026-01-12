alias b := build

all: build cp run

build:
  cargo build --target x86_64-unknown-uefi

run:
  qemu-system-x86_64 -enable-kvm \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp \
    -drive format=raw,file=rootfs.img \
    -serial stdio -m 1G

cp:
  cp ./target/x86_64-unknown-uefi/debug/plex.efi ./esp/efi/boot/bootx64.efi
