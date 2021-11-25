#!/usr/bin/bash

cargo build --release --target=wasm32-unknown-emscripten
cp ./target/wasm32-unknown-emscripten/release/shooting_rst.wasm ../shooting_gdt/