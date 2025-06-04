#!/bin/bash

pushd $(dirname $0)/..  # change to project root

cargo build

mkdir -p esp/EFI/BOOT
cp target/x86_64-unknown-uefi/debug/osproj.efi esp/EFI/BOOT/BOOTX64.EFI

exec qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp