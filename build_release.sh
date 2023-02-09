#!/bin/sh

##############################################################################
#
# Build the program for Linux and Windows, both with and without the
# `display_compat` feature.
# I don't provide a pre-built MacOSX binary because 
#  1) I don't have a Mac to test on
#  2) Apple makes it really annoying to cross-compile anything for their OS
#
# In particular, it creates 4 zip files, each containing a single
# `sots-event-inspect` binary built with the mentioned rustc target:
#  - windows.zip: x86_64-pc-windows-gnu
#  - windows-display-compat.zip: x86_64-pc-windows-gnu
#  - linux.zip: x86_64-unknown-linux-gnu
#  - linux-display-compat.zip: x86_64-unknown-linux-gnu
#
# Note: This script is intended to be run on a Linux system with mingw-w64
# installed to allow for Windows cross-compilation. It also needs to have the
# relevant toolchains installed using rustup:
#  - `rustup target add x86_64-pc-windows-gnu`
#  - `rustup target add x86_64-unknown-linux-gnu` (probably not needed on a Linux machine)

# Windows
echo "Building Windows"
cargo build --release --target x86_64-pc-windows-gnu
zip -j -9 release/windows.zip target/x86_64-pc-windows-gnu/release/sots-event-inspect.exe
zip -j -9 release/windows.zip README.md

echo "Building Windows (display_compat)"
cargo build --release --target x86_64-pc-windows-gnu --features display_compat
zip -j -9 release/windows-display-compat.zip target/x86_64-pc-windows-gnu/release/sots-event-inspect.exe
zip -j -9 release/windows-display-compat.zip README.md

# Linux
echo "Building Linux"
cargo build --release --target x86_64-unknown-linux-gnu
zip -j -9 release/linux.zip target/x86_64-unknown-linux-gnu/release/sots-event-inspect
zip -j -9 release/linux.zip README.md

echo "Building Linux (display_compat)"
cargo build --release --target x86_64-unknown-linux-gnu --features display_compat
zip -j -9 release/linux-display-compat.zip target/x86_64-unknown-linux-gnu/release/sots-event-inspect
zip -j -9 release/linux-display-compat.zip README.md

# Save SHA256 checksums to a file
echo "Saving checksums"
cd release
rm checksums.sha256
openssl dgst * > checksums.sha256
cd ..
