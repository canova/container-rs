#! /bin/bash

cargo build &&
sudo RUST_LOG=info ./target/debug/container-rs "$@"
