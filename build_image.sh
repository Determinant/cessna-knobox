#!/bin/bash
cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/cessna-knobox cessna-knobox.bin
