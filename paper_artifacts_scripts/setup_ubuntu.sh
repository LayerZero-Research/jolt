#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

MODE="${1:-native}"

usage() {
    cat <<'EOF'
Usage: bash paper_artifacts_scripts/setup_ubuntu.sh <native|docker>

Commands:
  native  Install Ubuntu packages, Rust toolchain, RISC-V targets, and jolt CLI.
  docker  Install Ubuntu packages and Docker Engine.
EOF
}

install_native() {
    sudo apt update
    sudo apt install -y build-essential pkg-config libssl-dev curl git python3 ca-certificates

    if ! command -v rustup >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    fi

    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi
    rustup toolchain install 1.94 \
        --profile minimal \
        --component cargo,rustc,clippy,rustfmt \
        --target riscv32imac-unknown-none-elf \
        --target riscv64imac-unknown-none-elf
    cargo install --path . --locked
}

install_docker() {
    sudo apt update
    sudo apt install -y ca-certificates curl git
    sudo install -m 0755 -d /etc/apt/keyrings
    sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
    sudo chmod a+r /etc/apt/keyrings/docker.asc
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}") stable" \
        | sudo tee /etc/apt/sources.list.d/docker.list >/dev/null
    sudo apt update
    sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    sudo docker run hello-world
    sudo usermod -aG docker "$USER"
    printf '\nDocker is installed. Start a new shell or run: newgrp docker\n'
}

case "$MODE" in
    native)
        install_native
        ;;
    docker)
        install_docker
        ;;
    -h|--help)
        usage
        ;;
    *)
        echo "Unknown setup mode: $MODE" >&2
        usage >&2
        exit 1
        ;;
esac
