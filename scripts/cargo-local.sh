#!/bin/sh
set -eu
repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export RUSTUP_HOME="$repo/.tooling/rustup"
export CARGO_HOME="$repo/.tooling/cargo"
export RUSTUP_TOOLCHAIN=1.90.0
cd "$repo"
exec cargo "$@"
