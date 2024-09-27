#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

TOOLPROOF=$(realpath "$SCRIPT_DIR/target/$1/toolproof")

cargo run --release -- --placeholders toolproof_path="$TOOLPROOF" -c 1
