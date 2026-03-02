#!/usr/bin/env bash
set -euo pipefail

ARCH="${1:-x86_64}"
KERNEL="./vmlinuz-${ARCH}"
ROOTFS="./rootfs-${ARCH}.img"

case "$ARCH" in
  x86_64)
    qemu-system-x86_64 -M pc -cpu host -enable-kvm \
      -kernel "$KERNEL" -initrd "$ROOTFS" \
      -append "console=ttyS0 root=/dev/ram0" \
      -nographic -m 256M -smp 2
    ;;
  aarch64)
    qemu-system-aarch64 -M virt -cpu cortex-a57 \
      -kernel "$KERNEL" -initrd "$ROOTFS" \
      -append "console=ttyAMA0 root=/dev/ram0" \
      -nographic -m 256M -smp 2
    ;;
  *)
    echo "unsupported ARCH: $ARCH" >&2
    exit 1
    ;;
esac
