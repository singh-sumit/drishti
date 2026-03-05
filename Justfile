set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

validate-skills:
    python3 scripts/validate_codex_skills.py

build:
    cargo build --workspace

build-ebpf:
    cargo run -p xtask -- build-ebpf

qemu-prepare arch="x86_64":
    cargo run -p xtask -- qemu prepare --arch {{arch}}

qemu-run arch="x86_64" kvm="auto" timeout="180":
    cargo run -p xtask -- qemu run --arch {{arch}} --kvm {{kvm}} --timeout-secs {{timeout}}

qemu-smoke arch="x86_64" kvm="auto" timeout="180":
    cargo run -p xtask -- qemu smoke --arch {{arch}} --kvm {{kvm}} --timeout-secs {{timeout}}

qemu-smoke-x86 timeout="180":
    cargo run -p xtask -- qemu smoke --arch x86_64 --kvm auto --timeout-secs {{timeout}}

qemu-smoke-arm64 timeout="300":
    cargo run -p xtask -- qemu smoke --arch aarch64 --kvm off --timeout-secs {{timeout}}

qemu-ci arch="x86_64" kvm="auto" timeout="180":
    cargo run -p xtask -- qemu ci --arch {{arch}} --kvm {{kvm}} --timeout-secs {{timeout}}

run:
    cargo run -p drishti-daemon -- --config config/drishti.toml

obs-up:
    #!/usr/bin/env bash
    set -euo pipefail

    require_cmd() {
      if ! command -v "$1" >/dev/null 2>&1; then
        echo "required command not found: $1" >&2
        exit 1
      fi
    }

    require_cmd docker
    require_cmd curl
    require_cmd ss

    if ! docker compose version >/dev/null 2>&1; then
      echo "docker compose is required but unavailable" >&2
      exit 1
    fi

    if ! sudo -n true 2>/dev/null; then
      echo "sudo credentials are required. Run: sudo -v" >&2
      exit 1
    fi

    state_dir="target/obs"
    pid_file="${state_dir}/drishti-daemon.pid"
    log_file="${state_dir}/drishti-daemon.log"
    repo_root="$(pwd)"
    log_file_abs="${repo_root}/${log_file}"
    max_series="${DRISHTI_EXPORT__MAX_SERIES:-50000}"

    mkdir -p "${state_dir}"

    cargo run -p xtask -- build-ebpf
    embedded_bpf_path="${repo_root}/target/bpfel-unknown-none/release/drishti-ebpf"
    if [[ ! -f "${embedded_bpf_path}" ]]; then
      echo "embedded eBPF artifact not found at ${embedded_bpf_path}" >&2
      exit 1
    fi

    DRISHTI_EMBEDDED_BPF_PATH="${embedded_bpf_path}" \
      cargo build -p drishti-daemon --features ebpf-runtime --bin drishti-daemon

    managed_pid=""
    if [[ -f "${pid_file}" ]]; then
      managed_pid="$(cat "${pid_file}" 2>/dev/null | tr -d '[:space:]' || true)"
      if [[ -z "${managed_pid}" ]]; then
        managed_pid="$(sudo cat "${pid_file}" 2>/dev/null | tr -d '[:space:]' || true)"
      fi

      if [[ "${managed_pid}" =~ ^[0-9]+$ ]] && sudo kill -0 "${managed_pid}" 2>/dev/null; then
        sudo kill "${managed_pid}" || true
        for _ in $(seq 1 20); do
          if ! sudo kill -0 "${managed_pid}" 2>/dev/null; then
            break
          fi
          sleep 0.25
        done
        if sudo kill -0 "${managed_pid}" 2>/dev/null; then
          sudo kill -9 "${managed_pid}" || true
        fi
      fi

      rm -f "${pid_file}" || sudo rm -f "${pid_file}"
    fi

    port_users="$(sudo ss -ltnp '( sport = :9090 )' 2>/dev/null | tail -n +2 || true)"
    if [[ -n "${port_users}" ]]; then
      echo "port 9090 is already in use by another process. Stop it before running obs-up." >&2
      echo "${port_users}" >&2
      exit 1
    fi

    sudo rm -f /tmp/drishti.pid
    sudo REPO_ROOT="${repo_root}" OBS_LOG_FILE="${log_file_abs}" EMBEDDED_BPF_PATH="${embedded_bpf_path}" OBS_MAX_SERIES="${max_series}" bash -c '
      DRISHTI_EMBEDDED_BPF_PATH="$EMBEDDED_BPF_PATH" \
      DRISHTI_DAEMON__PID_FILE=/tmp/drishti.pid \
      DRISHTI_EXPORT__MAX_SERIES="$OBS_MAX_SERIES" \
      DRISHTI_COLLECTORS__SYSCALL=true \
      nohup "$REPO_ROOT/target/debug/drishti-daemon" --config "$REPO_ROOT/config/drishti.toml" >"$OBS_LOG_FILE" 2>&1 < /dev/null &
    '

    daemon_pid=""
    for _ in $(seq 1 20); do
      if sudo test -f /tmp/drishti.pid; then
        daemon_pid="$(sudo cat /tmp/drishti.pid | tr -d '[:space:]' || true)"
        if [[ "${daemon_pid}" =~ ^[0-9]+$ ]] && sudo kill -0 "${daemon_pid}" 2>/dev/null; then
          break
        fi
      fi
      sleep 0.5
    done

    if [[ -z "${daemon_pid}" ]] || ! [[ "${daemon_pid}" =~ ^[0-9]+$ ]] || ! sudo kill -0 "${daemon_pid}" 2>/dev/null; then
      echo "failed to start drishti-daemon in managed mode" >&2
      if [[ -f "${log_file}" ]]; then
        sudo tail -n 50 "${log_file}" || true
      fi
      exit 1
    fi

    printf '%s\n' "${daemon_pid}" > "${pid_file}"

    daemon_ready=0
    for _ in $(seq 1 30); do
      if curl -fsS http://127.0.0.1:9090/healthz >/dev/null 2>&1; then
        daemon_ready=1
        break
      fi
      sleep 1
    done

    if [[ "${daemon_ready}" -ne 1 ]]; then
      echo "drishti-daemon did not become ready at http://127.0.0.1:9090/healthz within 30s" >&2
      if [[ -f "${log_file}" ]]; then
        sudo tail -n 50 "${log_file}" || true
      fi
      exit 1
    fi

    docker compose -f deploy/docker-compose.yml up -d --remove-orphans

    prometheus_ready=0
    for _ in $(seq 1 30); do
      if curl -fsS http://127.0.0.1:9091/-/ready >/dev/null 2>&1; then
        prometheus_ready=1
        break
      fi
      sleep 1
    done
    if [[ "${prometheus_ready}" -ne 1 ]]; then
      echo "prometheus did not become ready at http://127.0.0.1:9091/-/ready within 30s" >&2
      docker compose -f deploy/docker-compose.yml ps || true
      exit 1
    fi

    grafana_ready=0
    for _ in $(seq 1 30); do
      if curl -fsS http://127.0.0.1:3000/api/health >/dev/null 2>&1; then
        grafana_ready=1
        break
      fi
      sleep 1
    done
    if [[ "${grafana_ready}" -ne 1 ]]; then
      echo "grafana did not become ready at http://127.0.0.1:3000/api/health within 30s" >&2
      docker compose -f deploy/docker-compose.yml ps || true
      exit 1
    fi

    echo "Observability stack is up:"
    echo "  Drishti metrics: http://127.0.0.1:9090/metrics"
    echo "  Prometheus:      http://127.0.0.1:9091"
    echo "  Grafana:         http://127.0.0.1:3000 (admin/admin)"

obs-down:
    #!/usr/bin/env bash
    set -euo pipefail

    state_dir="target/obs"
    pid_file="${state_dir}/drishti-daemon.pid"

    if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
      docker compose -f deploy/docker-compose.yml down --remove-orphans || true
    else
      echo "docker compose is unavailable; skipping compose teardown"
    fi

    if [[ -f "${pid_file}" ]]; then
      daemon_pid="$(cat "${pid_file}" 2>/dev/null | tr -d '[:space:]' || true)"
      if [[ -z "${daemon_pid}" ]]; then
        daemon_pid="$(sudo cat "${pid_file}" 2>/dev/null | tr -d '[:space:]' || true)"
      fi

      if [[ "${daemon_pid}" =~ ^[0-9]+$ ]] && sudo kill -0 "${daemon_pid}" 2>/dev/null; then
        sudo kill "${daemon_pid}" || true
        for _ in $(seq 1 20); do
          if ! sudo kill -0 "${daemon_pid}" 2>/dev/null; then
            break
          fi
          sleep 0.25
        done
        if sudo kill -0 "${daemon_pid}" 2>/dev/null; then
          sudo kill -9 "${daemon_pid}" || true
        fi
      fi

      rm -f "${pid_file}" || sudo rm -f "${pid_file}"
    fi

    echo "Observability stack is down."

obs-status:
    #!/usr/bin/env bash
    set -uo pipefail

    state_dir="target/obs"
    pid_file="${state_dir}/drishti-daemon.pid"
    failed=0

    if [[ -f "${pid_file}" ]]; then
      daemon_pid="$(cat "${pid_file}" 2>/dev/null | tr -d '[:space:]' || true)"
      if [[ -z "${daemon_pid}" ]]; then
        daemon_pid="$(sudo cat "${pid_file}" 2>/dev/null | tr -d '[:space:]' || true)"
      fi

      if [[ "${daemon_pid}" =~ ^[0-9]+$ ]]; then
        if sudo -n kill -0 "${daemon_pid}" 2>/dev/null; then
          echo "drishti-daemon: up (pid ${daemon_pid})"
        else
          echo "drishti-daemon: down (managed pid ${daemon_pid} is not running or requires sudo -v)"
          failed=1
        fi
      else
        echo "drishti-daemon: down (invalid managed PID file: ${pid_file})"
        failed=1
      fi
    else
      echo "drishti-daemon: down (no managed PID file at ${pid_file})"
      failed=1
    fi

    check_url() {
      local name="$1"
      local url="$2"
      if curl -fsS "${url}" >/dev/null 2>&1; then
        echo "${name}: up (${url})"
      else
        echo "${name}: down (${url})"
        failed=1
      fi
    }

    check_url "drishti-healthz" "http://127.0.0.1:9090/healthz"
    check_url "prometheus-ready" "http://127.0.0.1:9091/-/ready"
    check_url "grafana-health" "http://127.0.0.1:3000/api/health"

    if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
      echo ""
      docker compose -f deploy/docker-compose.yml ps || failed=1
    else
      echo "docker compose: unavailable"
      failed=1
    fi

    if [[ "${failed}" -eq 0 ]]; then
      exit 0
    fi
    exit 1

docs-install:
    cd drishti-docs && npm ci

docs-dev:
    cd drishti-docs && npm run start

docs-build:
    cd drishti-docs && npm run build

docs-verify:
    cd drishti-docs && npm ci && npm run check:mermaid && npm run build
