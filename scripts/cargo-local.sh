#!/bin/sh
set -eu
repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export RUSTUP_HOME="$repo/.tooling/rustup"
export CARGO_HOME="$repo/.tooling/cargo"
export RUSTUP_TOOLCHAIN=1.90.0
cd "$repo"
# Homebrew 的 cargo 不读取 rustup 工具链选择，必须显式通过 rustup 启动。
exec rustup run "$RUSTUP_TOOLCHAIN" cargo "$@"
