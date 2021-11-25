#!/usr/bin/bash

cargo build --release --target=x86_64-pc-windows-gnu
cp ./target/x86_64-pc-windows-gnu/release/shooting_rst.dll ../shooting_gdt/