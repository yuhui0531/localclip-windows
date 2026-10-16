#!/bin/sh
set -eu
repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export RUSTUP_HOME="$repo/.tooling/rustup"
export CARGO_HOME="$repo/.tooling/cargo"
export RUSTUP_TOOLCHAIN=1.90.0
cd "$repo"
# cargo 的外部子命令也从 PATH 查找，避免混用 Homebrew 的 clippy/rustfmt。
localclip_toolchain_bin=$(dirname "$(rustup which --toolchain "$RUSTUP_TOOLCHAIN" cargo)")
export PATH="$localclip_toolchain_bin:$PATH"
exec cargo "$@"
