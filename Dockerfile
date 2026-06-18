FROM rust:1-trixie AS builder

WORKDIR /workdir

# Compile the project
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/workdir/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo install --locked --path .


FROM debian:trixie-slim AS runtime

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

RUN apt update && \
    apt install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /opt/openai-completions-proxy
COPY --from=builder /usr/local/cargo/bin/openai-completions-proxy /usr/local/bin/openai-completions-proxy

CMD ["openai-completions-proxy"]
