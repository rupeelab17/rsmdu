#!/bin/bash

# Script to fix wasm32-unknown-unknown target installation issues

set -e

echo "🔧 Fixing wasm32-unknown-unknown target installation..."

# Ensure we use rustup instead of Homebrew Rust
export PATH="$HOME/.cargo/bin:$PATH"

# Set rustup environment
export RUSTUP_HOME=~/.rustup
export CARGO_HOME=~/.cargo

# Create necessary directories
echo "📁 Creating rustup cache directories..."
mkdir -p ~/.rustup/tmp
mkdir -p ~/.rustup/downloads
mkdir -p ~/.rustup/update-hashes
chmod 755 ~/.rustup/tmp
chmod 755 ~/.rustup/downloads
chmod 755 ~/.rustup/update-hashes

# Check if rustup is available
if ! command -v rustup &> /dev/null; then
    echo "❌ rustup is not installed."
    echo ""
    echo "Please install rustup:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check current toolchain
echo "🔍 Current Rust toolchain:"
rustup show

# Try to add wasm32-unknown-unknown target
echo ""
echo "📥 Installing wasm32-unknown-unknown target..."
if rustup target add wasm32-unknown-unknown; then
    echo "✅ Successfully installed wasm32-unknown-unknown target!"
else
    echo "❌ Failed to install wasm32-unknown-unknown target."
    echo ""
    echo "Try the following:"
    echo "  1. Check your internet connection"
    echo "  2. Update rustup: brew upgrade rustup (if using Homebrew)"
    echo "  3. Or reinstall rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Verify installation
echo ""
echo "✅ Verifying installation..."
if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "✅ wasm32-unknown-unknown target is installed and ready!"
else
    echo "⚠️  Target may not be properly installed. Try running:"
    echo "   rustup target list --installed"
    exit 1
fi

