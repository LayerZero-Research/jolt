FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:${PATH}

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        build-essential \
        ca-certificates \
        curl \
        git \
        libssl-dev \
        pkg-config \
        python3 \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain none \
    && rustup toolchain install 1.94 \
        --profile minimal \
        --component cargo,rustc,clippy,rustfmt \
        --target riscv32imac-unknown-none-elf \
        --target riscv64imac-unknown-none-elf

WORKDIR /artifact
COPY . .

RUN cargo --version \
    && rustc --version \
    && cargo install --path . --locked \
    && cargo build --release -p fibonacci \
    && cargo build --release -p sha2-chain \
    && cargo build --release -p recursion --bin recursion

ENTRYPOINT ["bash", "paper_artifacts_scripts/reproduce_paper_experiments.sh"]
CMD ["--help"]
