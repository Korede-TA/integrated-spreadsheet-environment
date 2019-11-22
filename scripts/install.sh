#!/bin/bash

set -ex

## install npm and cargo (rust) dependencies
echo "install npm and cargo dependencies"
[[ -f "$(which npm)" ]] && npm install
# npm deps include: 
# - electron
# - webpack

[[ -f "$(which cargo)" ]] && cargo install
# cargo deps include: 
# - wasm-bindgen
# - cargo-web
# - cargo-watch

# PATCH: re-install wasm-bindgen-cli from git repo as patched version
# helps avoid nasty bugs (see: https://github.com/rustwasm/wasm-bindgen/issues/857)
[[ -f "$(which cargo)" ]] && cargo install wasm-bindgen-cli --git https://github.com/rustwasm/wasm-bindgen --force
