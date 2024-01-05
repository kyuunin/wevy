export RUST_BACKTRACE := "1"

list:
    just --list

setup-ubuntu:
    sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0

setup-wasm:
    cargo install wasm-server-runner

run:
    cargo run

run-wasm:
    cargo run --target wasm32-unknown-unknown

lint:
    cargo clippy

build:
    cargo build
