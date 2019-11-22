#!/bin/bash

set -ex

DIR="$(dirname $0)"
WASM_TARGET="$DIR/wasm32-unknown-unknown"
WASM_NAME="integrated_spreadsheet_environment"
WASM_NAME=""$(cat "$DIR/Cargo.toml" | grep name | sed 's/name = "//' | sed 's/"//g'| sed 's/-/_/g')""
APP_DIR="$DIR/static"
BUILD_DIR="$DIR/dist"
ENV="development"

usage() {
    echo `./build.sh $0`: ERROR: $* 1>&2
    echo usage: `./basename $0` '[--cargo-web] [--wasm-pack] [--bindgen-webpack]
        [file ...]' 1>&2
    exit 1
}

ARTIFACTS_DIR="target/$WASM_TARGET/debug" # default artifacts dir

## Building alternatives:
case "$1" in
  --cargo-web) 
    # with `cargo-web`: .js and .wasm artifacts stored at target/$WASM_TARGET/debug/
    echo "building with 'cargo web'"
    [[ -f "$(which cargo-web)" ]] && cargo-web build --target=wasm32-unknown-unknown
    ;;
  --wasm-pack)
    # with `wasm-pack`: builds final .wasm and .js to pkg/ directory, with .js as ES6 module
    #  which are hard to import on the web.
    echo "building with 'wasm pack'"
    [[ -f "$(which wasm-pack)" ]] && wasm-pack build
    ARTIFACTS_DIR="pkg"
    ;;
  --bindgen-webpack) 
    # with a combination of `cargo build && wasm-bindgen && webpack`
    # `cargo build --target=wasm32-unknown-unknown`
    # works closely to cargo-web, same build directory but with only .wasm.
    # `wasm-bindgen` has to also be used to generate .js
    # (see example here: 
    #   https://github.com/anderejd/electron-wasm-rust-example/blob/a84437735c/build.sh)
    echo "building wth 'cargo build' && 'wasm-bindgen'"
    [[ -f "$(which cargo)" ]] && cargo build --target=wasm32-unknown-unknown
    [[ -f "$(which wasm-bindgen)" ]] && \
      wasm-bindgen "target/$WASM_TARGET/debug/$WASM_NAME.wasm" --out-dir "$APP_DIR" --no-typescript && \
      "$DIR/node_modules/webpack-cli/bin/cli.js" --mode=$ENV"$APP_DIR/app_loader.js" -o "$APP_DIR/bundle.js"
    ;;
  -*) usage "bad argument $1";;
  *) break;;
esac

mkdir -p $BUILD_DIR
cp "$ARTIFACTS_DIR/$WASM_NAME.js" $BUILD_DIR/
cp "$ARTIFACTS_DIR/$WASM_NAME.wasm" $BUILD_DIR/
