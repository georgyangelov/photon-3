#!/bin/sh

#TARGET=wasm32-wasi
TARGET=wasm32-unknown-unknown

cd "$(dirname "$0")/.."

cargo build -p runtime --target="$TARGET" --release

wasm2wat target/"$TARGET"/release/runtime.wasm -o target/"$TARGET"/release/runtime.wat
