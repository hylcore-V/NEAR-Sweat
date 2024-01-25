#!/bin/bash
set -eox pipefail

echo ">> Building stub contract"

rustup target add wasm32-unknown-unknown
cargo build -p exploit-stub --target wasm32-unknown-unknown --profile=contract

cp ./target/wasm32-unknown-unknown/contract/exploit_stub.wasm res/exploit_stub.wasm
