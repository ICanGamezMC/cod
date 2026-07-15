#!/usr/bin/env bash
set -e

echo "=== 1. Installing Rust Targets ==="
rustup target add x86_64-unknown-linux-musl    # Linux (Static)
rustup target add x86_64-pc-windows-gnu        # Windows
rustup target add x86_64-apple-darwin          # macOS (Intel)
rustup target add aarch64-apple-darwin         # macOS (Apple Silicon)

echo "=== 2. Installing Zig and cargo-zigbuild ==="
sudo apt update && sudo apt install -y curl tar
if ! command -v zig &> /dev/null; then
    sudo snap install zig --classic --beta
fi
if ! command -v cargo-zigbuild &> /dev/null; then
    cargo install --locked cargo-zigbuild
fi

echo "=== 3. Fetching macOS SDK (if not present) ==="
if [ ! -d "MacOSX13.1.sdk" ]; then
    curl -L https://github.com/roblabla/MacOSX-SDKs/releases/download/13.1/MacOSX13.1.sdk.tar.xz | tar xJ
fi
export SDKROOT="$(pwd)/MacOSX13.1.sdk"

echo "=== 4. Compiling Binaries ==="
# Determine binary name from Cargo.toml
BIN_NAME=$(grep -m 1 '^name =' Cargo.toml | cut -d '"' -f 2)
if [ -z "$BIN_NAME" ]; then
    # Fallback if parsing fails
    BIN_NAME="my-cli"
fi

echo "Building for Linux (static)..."
cargo zigbuild --release --target x86_64-unknown-linux-musl

echo "Building for Windows..."
cargo zigbuild --release --target x86_64-pc-windows-gnu

echo "Building for macOS (Apple Silicon)..."
cargo zigbuild --release --target aarch64-apple-darwin

echo "Building for macOS (Intel)..."
cargo zigbuild --release --target x86_64-apple-darwin

echo "=== 5. Organizing Outputs ==="
mkdir -p dist
cp "target/x86_64-unknown-linux-musl/release/${BIN_NAME}" "dist/${BIN_NAME}-linux-amd64"
cp "target/x86_64-pc-windows-gnu/release/${BIN_NAME}.exe" "dist/${BIN_NAME}-windows-amd64.exe"
cp "target/aarch64-apple-darwin/release/${BIN_NAME}" "dist/${BIN_NAME}-macos-arm64"
cp "target/x86_64-apple-darwin/release/${BIN_NAME}" "dist/${BIN_NAME}-macos-amd64"

echo "=== Done! Your compiled binaries are in the ./dist/ folder: ==="
ls -l dist/