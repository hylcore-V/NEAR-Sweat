#!/bin/bash
set -eox pipefail

echo ">> Building stub contract"

rustup target add wasm32-unknown-unknown
cargo build -p defer-stub --target wasm32-unknown-unknown --profile=contract

cp ./target/wasm32-unknown-unknown/contract/defer_stub.wasm res/defer_stub.wasm
