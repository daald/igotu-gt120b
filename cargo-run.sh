#!/bin/sh

set -ex

#cargo install --path .
cargo build

./target/debug/bulk-test2
