#!/usr/bin/bash

export EMMAKEN_CFLAGS="-s SIDE_MODULE=1 -shared -Wl,--no-check-features -all"
export C_INCLUDE_PATH="$EMSDK/upstream/emscripten/cache/sysroot/include/"

cargo build --release --target=wasm32-unknown-emscripten
cp ./target/wasm32-unknown-emscripten/release/shooting_rst.wasm ../shooting_gdt/