#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

ARCH="x86_64"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --arch)
      ARCH="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" ]]; then
  echo "unsupported arch: $ARCH" >&2
  exit 1
fi

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

is_readable_file() {
  local path="$1"
  [[ -n "$path" && -f "$path" && -r "$path" ]]
}

require_cmd() {
  if ! command_exists "$1"; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_cmd cpio
require_cmd gzip
require_cmd file
require_cmd python3
require_cmd busybox

if [[ "$ARCH" == "x86_64" ]]; then
  require_cmd qemu-system-x86_64
else
  require_cmd qemu-system-aarch64
fi

ARTIFACT_ROOT="${DRISHTI_QEMU_ARTIFACT_ROOT:-$REPO_ROOT/target/qemu}"
ARCH_DIR="$ARTIFACT_ROOT/$ARCH"
INITRAMFS_DIR="$ARCH_DIR/initramfs"
KERNEL_OUT="$ARCH_DIR/vmlinuz"
INITRD_OUT="$ARCH_DIR/initramfs.cpio.gz"

mkdir -p "$ARCH_DIR" "$INITRAMFS_DIR"

resolve_target_triple() {
  if [[ "$ARCH" == "x86_64" ]]; then
    echo "x86_64-unknown-linux-gnu"
  else
    echo "aarch64-unknown-linux-musl"
  fi
}

TARGET_TRIPLE="$(resolve_target_triple)"
DEFAULT_DAEMON_BIN="$REPO_ROOT/target/$TARGET_TRIPLE/release/drishti-daemon"
DEFAULT_SMOKE_BIN="$REPO_ROOT/target/$TARGET_TRIPLE/release/qemu_smoke"

DAEMON_BIN="${DRISHTI_QEMU_DAEMON_BIN:-$DEFAULT_DAEMON_BIN}"
SMOKE_BIN="${DRISHTI_QEMU_SMOKE_BIN:-$DEFAULT_SMOKE_BIN}"

if [[ ! -f "$DAEMON_BIN" ]]; then
  echo "missing daemon binary: $DAEMON_BIN" >&2
  echo "hint: cargo build -p drishti-daemon --features ebpf-runtime --release --target $TARGET_TRIPLE --bin drishti-daemon" >&2
  exit 1
fi

if [[ ! -f "$SMOKE_BIN" ]]; then
  echo "missing qemu smoke binary: $SMOKE_BIN" >&2
  echo "hint: cargo build -p drishti-daemon --features ebpf-runtime --release --target $TARGET_TRIPLE --bin qemu_smoke" >&2
  exit 1
fi

copy_runtime_deps() {
  local binary="$1"

  local file_output
  file_output="$(file "$binary" || true)"
  if echo "$file_output" | grep -q "statically linked"; then
    return
  fi

  while IFS= read -r lib; do
    [[ -z "$lib" ]] && continue
    if [[ ! -f "$lib" ]]; then
      continue
    fi

    local dest="$INITRAMFS_DIR$lib"
    mkdir -p "$(dirname "$dest")"
    cp -L "$lib" "$dest"
  done < <(ldd "$binary" 2>/dev/null | awk '
    $2 == "=>" && $3 ~ /^\// { print $3 }
    $1 ~ /^\// { print $1 }
  ' | sort -u)
}

extract_x86_kernel_from_deb() {
  local deb_path="$1"
  local extract_dir="$ARCH_DIR/kernel-extract"

  require_cmd dpkg-deb
  rm -rf "$extract_dir"
  mkdir -p "$extract_dir"
  dpkg-deb -x "$deb_path" "$extract_dir"

  local extracted
  extracted="$(find "$extract_dir/boot" -maxdepth 1 -type f -name 'vmlinuz-*' | sort | head -n 1 || true)"
  if [[ -z "$extracted" ]]; then
    echo ""
    return
  fi

  echo "$extracted"
}

download_kernel_deb() {
  local package_name="$1"
  local download_dir="$ARCH_DIR/kernel-deb"

  require_cmd apt-get
  mkdir -p "$download_dir"

  (
    cd "$download_dir"
    rm -f ./*.deb
    apt-get download "$package_name" >/dev/null
  )

  ls -1 "$download_dir"/"${package_name}"_*.deb 2>/dev/null | head -n 1 || true
}

resolve_x86_kernel_package() {
  require_cmd apt-cache

  local package_name
  package_name="$(apt-cache depends linux-image-generic 2>/dev/null | awk '/Depends: linux-image-[0-9].*-generic/{print $2; exit}')"
  if [[ -n "$package_name" ]]; then
    echo "$package_name"
    return
  fi

  package_name="$(apt-cache search '^linux-image-[0-9].*-generic$' 2>/dev/null | awk '{print $1}' | sort -Vr | head -n 1 || true)"
  echo "$package_name"
}

resolve_x86_kernel() {
  if [[ -n "${DRISHTI_QEMU_X86_64_KERNEL:-}" ]]; then
    if is_readable_file "$DRISHTI_QEMU_X86_64_KERNEL"; then
      echo "$DRISHTI_QEMU_X86_64_KERNEL"
      return
    fi

    echo "configured DRISHTI_QEMU_X86_64_KERNEL is missing or unreadable: $DRISHTI_QEMU_X86_64_KERNEL" >&2
    echo ""
    return
  fi

  local host_kernel="/boot/vmlinuz-$(uname -r)"
  if is_readable_file "$host_kernel"; then
    echo "$host_kernel"
    return
  fi

  local fallback
  fallback="$(find /boot -maxdepth 1 -type f -name 'vmlinuz*' -readable 2>/dev/null | sort | head -n 1 || true)"
  if is_readable_file "$fallback"; then
    echo "$fallback"
    return
  fi

  if [[ -n "${DRISHTI_QEMU_X86_64_KERNEL_DEB:-}" ]]; then
    extract_x86_kernel_from_deb "$DRISHTI_QEMU_X86_64_KERNEL_DEB"
    return
  fi

  local kernel_pkg
  kernel_pkg="$(resolve_x86_kernel_package)"
  if [[ -n "$kernel_pkg" ]]; then
    local kernel_deb
    kernel_deb="$(download_kernel_deb "$kernel_pkg")"
    if [[ -n "$kernel_deb" && -f "$kernel_deb" ]]; then
      extract_x86_kernel_from_deb "$kernel_deb"
      return
    fi
  fi

  echo ""
}

extract_arm64_kernel_from_deb() {
  local deb_path="$1"
  local extract_dir="$ARCH_DIR/kernel-extract"

  require_cmd dpkg-deb
  rm -rf "$extract_dir"
  mkdir -p "$extract_dir"
  dpkg-deb -x "$deb_path" "$extract_dir"

  local extracted
  extracted="$(find "$extract_dir/boot" -maxdepth 1 -type f -name 'vmlinuz-*' | sort | head -n 1 || true)"
  if [[ -z "$extracted" ]]; then
    echo ""
    return
  fi

  echo "$extracted"
}

resolve_aarch64_kernel() {
  if [[ -n "${DRISHTI_QEMU_AARCH64_KERNEL:-}" ]]; then
    echo "$DRISHTI_QEMU_AARCH64_KERNEL"
    return
  fi

  if [[ -n "${DRISHTI_QEMU_AARCH64_KERNEL_DEB:-}" ]]; then
    extract_arm64_kernel_from_deb "$DRISHTI_QEMU_AARCH64_KERNEL_DEB"
    return
  fi

  echo ""
}

if [[ "$ARCH" == "x86_64" ]]; then
  KERNEL_SRC="$(resolve_x86_kernel)"
else
  KERNEL_SRC="$(resolve_aarch64_kernel)"
fi

if [[ -z "$KERNEL_SRC" || ! -f "$KERNEL_SRC" ]]; then
  if [[ "$ARCH" == "aarch64" ]]; then
    cat >&2 <<MSG
unable to locate aarch64 kernel image.
Provide one of:
  DRISHTI_QEMU_AARCH64_KERNEL=/abs/path/to/vmlinuz
  DRISHTI_QEMU_AARCH64_KERNEL_DEB=/abs/path/to/linux-image-arm64.deb
MSG
  else
    cat >&2 <<MSG
unable to locate x86_64 kernel image.
Provide one of:
  DRISHTI_QEMU_X86_64_KERNEL=/abs/path/to/vmlinuz
  DRISHTI_QEMU_X86_64_KERNEL_DEB=/abs/path/to/linux-image-amd64.deb
MSG
  fi
  exit 1
fi

rm -rf "$INITRAMFS_DIR"
mkdir -p "$INITRAMFS_DIR/bin" "$INITRAMFS_DIR/etc" "$INITRAMFS_DIR/proc" "$INITRAMFS_DIR/sys" "$INITRAMFS_DIR/tmp"

cp "$SMOKE_BIN" "$INITRAMFS_DIR/init"
cp "$DAEMON_BIN" "$INITRAMFS_DIR/bin/drishti-daemon"
chmod +x "$INITRAMFS_DIR/init" "$INITRAMFS_DIR/bin/drishti-daemon"

if command_exists busybox; then
  cp -L "$(command -v busybox)" "$INITRAMFS_DIR/bin/busybox"
  chmod +x "$INITRAMFS_DIR/bin/busybox"
fi

copy_runtime_deps "$SMOKE_BIN"
copy_runtime_deps "$DAEMON_BIN"

cat > "$INITRAMFS_DIR/etc/drishti.toml" <<CFG
[daemon]
pid_file = "/tmp/drishti.pid"
log_level = "info"
metrics_addr = "127.0.0.1:9090"

[collectors]
cpu = true

[collectors.process]
enabled = true
track_threads = false

[collectors.memory]
enabled = true
poll_interval_ms = 1000
track_oom = true

[collectors.network]
enabled = true
interfaces = []
tcp_rtt = true
tcp_retransmits = true

[collectors.disk]
enabled = true
devices = []
latency_buckets_usec = [10, 50, 100, 500, 1000, 5000, 10000]

[collectors.syscall]
enabled = true
top_n = 20
latency_buckets_usec = [1, 10, 50, 100, 500, 1000, 5000]

[filters]
exclude_pids = []
exclude_comms = []
include_comms = []

[export]
scrape_interval_ms = 1000
max_series = 10000
CFG

cp "$KERNEL_SRC" "$KERNEL_OUT"

(
  cd "$INITRAMFS_DIR"
  find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "$INITRD_OUT"
)

echo "prepared QEMU artifacts"
echo "  arch: $ARCH"
echo "  kernel: $KERNEL_OUT"
echo "  initramfs: $INITRD_OUT"
echo "  daemon-bin: $DAEMON_BIN"
echo "  smoke-bin: $SMOKE_BIN"
