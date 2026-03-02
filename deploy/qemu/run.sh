#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

ARCH="x86_64"
KVM_MODE="auto"
TIMEOUT_SECS=180
SERIAL_LOG_OVERRIDE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --arch)
      ARCH="${2:-}"
      shift 2
      ;;
    --kvm)
      KVM_MODE="${2:-}"
      shift 2
      ;;
    --timeout)
      TIMEOUT_SECS="${2:-}"
      shift 2
      ;;
    --serial-log)
      SERIAL_LOG_OVERRIDE="${2:-}"
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

if [[ "$KVM_MODE" != "auto" && "$KVM_MODE" != "on" && "$KVM_MODE" != "off" ]]; then
  echo "unsupported kvm mode: $KVM_MODE" >&2
  exit 1
fi

ARTIFACT_ROOT="${DRISHTI_QEMU_ARTIFACT_ROOT:-$REPO_ROOT/target/qemu}"
ARCH_DIR="$ARTIFACT_ROOT/$ARCH"
KERNEL="$ARCH_DIR/vmlinuz"
INITRD="$ARCH_DIR/initramfs.cpio.gz"
SERIAL_LOG="$ARCH_DIR/serial.log"
SMOKE_LOG="$ARCH_DIR/smoke.log"
METRICS_OUT="$ARCH_DIR/metrics.prom"
SUMMARY_OUT="$ARCH_DIR/summary.json"

if [[ -n "$SERIAL_LOG_OVERRIDE" ]]; then
  SERIAL_LOG="$SERIAL_LOG_OVERRIDE"
fi

if [[ ! -f "$KERNEL" || ! -f "$INITRD" ]]; then
  echo "missing QEMU artifacts for $ARCH; run deploy/qemu/prepare.sh --arch $ARCH first" >&2
  exit 1
fi

mkdir -p "$ARCH_DIR"
mkdir -p "$(dirname "$SERIAL_LOG")"
: > "$SERIAL_LOG"
: > "$SMOKE_LOG"
: > "$METRICS_OUT"

is_kvm_available() {
  [[ -c /dev/kvm && -r /dev/kvm && -w /dev/kvm ]]
}

should_enable_kvm() {
  if [[ "$KVM_MODE" == "on" ]]; then
    return 0
  fi
  if [[ "$KVM_MODE" == "off" ]]; then
    return 1
  fi
  is_kvm_available
}

build_qemu_cmd() {
  local -n cmd_ref=$1
  cmd_ref=()

  if [[ "$ARCH" == "x86_64" ]]; then
    cmd_ref=(
      qemu-system-x86_64
      -M pc
      -m 768M
      -smp 2
      -nographic
      -no-reboot
      -kernel "$KERNEL"
      -initrd "$INITRD"
      -append "console=ttyS0 rdinit=/init panic=-1 devtmpfs.mount=1"
    )

    if should_enable_kvm; then
      cmd_ref+=( -enable-kvm -cpu host )
    else
      cmd_ref+=( -cpu max )
    fi
  else
    cmd_ref=(
      qemu-system-aarch64
      -M virt
      -cpu cortex-a57
      -m 768M
      -smp 2
      -nographic
      -no-reboot
      -kernel "$KERNEL"
      -initrd "$INITRD"
      -append "console=ttyAMA0 rdinit=/init panic=-1 devtmpfs.mount=1"
    )
  fi
}

extract_metrics() {
  sed -E 's/\x1B\[[0-9;]*[A-Za-z]//g' "$SERIAL_LOG" \
    | tr -d '\r' \
    | awk '
      index($0, "DRISHTI_METRICS_BEGIN") { collect=1; next }
      index($0, "DRISHTI_METRICS_END") { collect=0; next }
      collect { print }
    ' > "$METRICS_OUT" || true
}

write_summary() {
  local result="$1"
  local error_category="$2"
  local message="$3"

  local summary_line
  summary_line="$(grep -m1 '^DRISHTI_QEMU_SUMMARY=' "$SERIAL_LOG" || true)"
  if [[ -n "$summary_line" ]]; then
    printf '%s\n' "${summary_line#DRISHTI_QEMU_SUMMARY=}" > "$SUMMARY_OUT"
    return
  fi

  cat > "$SUMMARY_OUT" <<JSON
{"result":"$result","error_category":"$error_category","message":"$message"}
JSON
}

QEMU_CMD=()
build_qemu_cmd QEMU_CMD

echo "starting QEMU (${ARCH})"
printf 'command: %q ' "${QEMU_CMD[@]}"
printf '\n'

"${QEMU_CMD[@]}" >"$SERIAL_LOG" 2>&1 &
QEMU_PID=$!

RESULT=""
DEADLINE=$(( $(date +%s) + TIMEOUT_SECS ))

while kill -0 "$QEMU_PID" 2>/dev/null; do
  if grep -q '^DRISHTI_QEMU_RESULT=PASS' "$SERIAL_LOG"; then
    RESULT="PASS"
    break
  fi

  if grep -q '^DRISHTI_QEMU_RESULT=FAIL' "$SERIAL_LOG"; then
    RESULT="FAIL"
    break
  fi

  if (( $(date +%s) >= DEADLINE )); then
    RESULT="TIMEOUT"
    break
  fi

  sleep 1
done

if kill -0 "$QEMU_PID" 2>/dev/null; then
  kill "$QEMU_PID" >/dev/null 2>&1 || true
fi
wait "$QEMU_PID" >/dev/null 2>&1 || true

# QEMU can exit quickly after init terminates; do one final marker scan to
# avoid losing PASS/FAIL status due a race with the loop condition.
if [[ -z "$RESULT" ]]; then
  if grep -q '^DRISHTI_QEMU_RESULT=PASS' "$SERIAL_LOG"; then
    RESULT="PASS"
  elif grep -q '^DRISHTI_QEMU_RESULT=FAIL' "$SERIAL_LOG"; then
    RESULT="FAIL"
  fi
fi

cp "$SERIAL_LOG" "$SMOKE_LOG"
extract_metrics

case "$RESULT" in
  PASS)
    write_summary "PASS" "" "QEMU smoke completed successfully"
    echo "QEMU smoke PASS ($ARCH)"
    exit 0
    ;;
  FAIL)
    write_summary "FAIL" "guest_smoke_failure" "Guest smoke binary reported failure"
    echo "QEMU smoke FAIL ($ARCH)" >&2
    exit 1
    ;;
  TIMEOUT)
    write_summary "FAIL" "boot_timeout" "Timed out waiting for guest smoke markers"
    echo "QEMU smoke TIMEOUT ($ARCH)" >&2
    exit 124
    ;;
  *)
    write_summary "FAIL" "guest_exit_without_result" "QEMU exited before producing smoke markers"
    echo "QEMU exited without result markers ($ARCH)" >&2
    exit 1
    ;;
esac
