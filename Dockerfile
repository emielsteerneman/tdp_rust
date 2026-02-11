# Stage 1: Build Rust backend
FROM ubuntu:24.04 AS builder

RUN apt-get update && apt-get install -y curl build-essential libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Root workspace crate has [package] but no src/ — create stub so cargo parses it
RUN mkdir src && echo "" > src/lib.rs
COPY api/ api/
COPY chat/ chat/
COPY configuration/ configuration/
COPY data_access/ data_access/
COPY data_processing/ data_processing/
COPY data_structures/ data_structures/
COPY mcp/ mcp/
COPY pipeline/ pipeline/
COPY web/ web/

RUN cargo build -p web

# Stage 2: Runtime with Node for frontend dev server
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y curl && \
    curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust binary and ONNX Runtime shared lib
COPY --from=builder /app/target/debug/web /app/web
COPY --from=builder /app/target/debug/deps/libonnxruntime.so* /app/
ENV LD_LIBRARY_PATH="/app"

# Install frontend dependencies
COPY frontend/ /app/frontend/
RUN cd /app/frontend && npm ci

# Copy entrypoint
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
