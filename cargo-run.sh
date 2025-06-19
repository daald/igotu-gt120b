#!/bin/sh

set -ex

# sudo apt-get install cargo rustfmt


#cargo install --path .
cargo build

RUST_BACKTRACE=1 ./target/debug/igotu-gt120
