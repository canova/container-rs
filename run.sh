#! /bin/bash

sudo -u canova /home/canova/.cargo/bin/cargo build &&
./target/debug/container-rs "$@"