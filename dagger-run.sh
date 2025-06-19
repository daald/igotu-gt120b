#!/bin/sh

set -ex

dagger -c 'build | export target'

RUST_BACKTRACE=1 ./target/debug/igotu-gt120
