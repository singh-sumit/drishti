#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

ARCH="x86_64"
KVM_MODE="auto"
TIMEOUT_SECS=180
SKIP_PREPARE=0

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
    --skip-prepare)
      SKIP_PREPARE=1
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if (( SKIP_PREPARE == 0 )); then
  "$SCRIPT_DIR/prepare.sh" --arch "$ARCH"
fi

set +e
"$SCRIPT_DIR/run.sh" --arch "$ARCH" --kvm "$KVM_MODE" --timeout "$TIMEOUT_SECS"
status=$?
set -e

ARTIFACT_ROOT="${DRISHTI_QEMU_ARTIFACT_ROOT:-$REPO_ROOT/target/qemu}"
ARCH_DIR="$ARTIFACT_ROOT/$ARCH"
SUMMARY_OUT="$ARCH_DIR/summary.json"
SERIAL_LOG="$ARCH_DIR/serial.log"
METRICS_OUT="$ARCH_DIR/metrics.prom"

if [[ -f "$SUMMARY_OUT" ]]; then
  echo "QEMU summary ($ARCH):"
  cat "$SUMMARY_OUT"
else
  echo "QEMU summary file missing: $SUMMARY_OUT" >&2
fi

if [[ -f "$METRICS_OUT" ]]; then
  echo "QEMU metrics snapshot lines: $(wc -l < "$METRICS_OUT")"
fi

if [[ -f "$SERIAL_LOG" ]]; then
  echo "QEMU serial log tail ($ARCH):"
  tail -n 40 "$SERIAL_LOG" || true
fi

exit "$status"
