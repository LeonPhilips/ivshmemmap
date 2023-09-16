#!/bin/bash
# Designed for use with x86_64-pc-windows-gnu
# Example usage:  ./cross_compile.sh x86_64-pc-windows-gnu /path/to/copy/result/to/

# Delete file if it exists
rm "$2"
set -e

# We compile our target in a different directory to prevent cargo from messing up incremental builds
cross build --target "$1" --release --target-dir "target/$1-cross" && cp "./target/$1-cross/$1/release/ivshmemmap-tests.exe" "$2"

# Notify that we're done.
paplay /usr/share/sounds/freedesktop/stereo/dialog-information.oga && paplay /usr/share/sounds/freedesktop/stereo/dialog-information.oga