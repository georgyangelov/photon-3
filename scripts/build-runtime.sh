#!/bin/sh

TARGET=wasm32-wasi

cd "$(dirname "$0")/.."

cargo build -p runtime --target="$TARGET" --release

wasm2wat target/"$TARGET"/release/runtime.wasm -o target/"$TARGET"/release/runtime.wat
