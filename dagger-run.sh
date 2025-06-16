#!/bin/sh

set -ex

dagger -c 'build | export target'

./target/release/bulk-test2
