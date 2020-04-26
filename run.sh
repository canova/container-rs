#! /bin/bash

sudo -u canova /home/canova/.cargo/bin/cargo build &&
RUST_LOG=info ./target/debug/container-rs "$@"