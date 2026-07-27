#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")"

cargo clean
RUSTFLAGS="${RUSTFLAGS:-} -C target-cpu=native" cargo build --release
