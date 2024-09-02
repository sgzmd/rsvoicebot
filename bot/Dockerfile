FROM rust:1.72 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libssl-dev \
    pkg-config \
    clang \
    make \
    cmake \
    && rm -rf /var/lib/apt/lists/*

RUN USER=root cargo new --bin app
WORKDIR /app

COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

ENV GGML=/ggml-base.en.bin
ENV RUST_LOG=info
ENV RECORDING_TO_WALL_RATIO=8
ENV WHISPER_THREADS=8

COPY --from=builder /app/target/release/voicebot /usr/local/bin/voicebot

RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg wget ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN wget --show-progress \
    -O /ggml-base.en.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin

CMD ["voicebot"]