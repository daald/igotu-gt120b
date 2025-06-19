#!/bin/sh

set -ex

# sudo apt-get install cargo rustfmt


#cargo install --path .
cargo build

./target/debug/bulk-test2
