FROM rust:1.72 AS builder

RUN USER=root cargo new --bin app
WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src

RUN cargo build --release

FROM debian:buster-slim
