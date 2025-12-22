# Installing wasm32-unknown-unknown Target

If you encounter the error:

```
Error: wasm32-unknown-unknown target not found in sysroot
```

## Solution 1: Use rustup (Recommended)

If you have rustup installed:

```bash
rustup target add wasm32-unknown-unknown
```

If you get cache errors, try:

```bash
# Create cache directory
mkdir -p ~/.rustup/tmp
chmod 755 ~/.rustup/tmp

# Try again
rustup target add wasm32-unknown-unknown
```

## Solution 2: Switch to rustup

If you're using Homebrew Rust, switch to rustup:

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell
source ~/.cargo/env

# Install wasm target
rustup target add wasm32-unknown-unknown
```

## Solution 3: Manual Installation (Advanced)

For non-rustup setups, see:
https://rustwasm.github.io/wasm-pack/book/prerequisites/non-rustup-setups.html

## Verify Installation

Check if the target is installed:

```bash
rustup target list --installed | grep wasm32-unknown-unknown
```

Or:

```bash
rustc --print target-list | grep wasm32
```
