# Installation Guide

## From Crates.io (Recommended)

Once published, install with:

```bash
cargo install rpg
```

This will install the `rpg` binary to `~/.cargo/bin/rpg` (or equivalent on your system).

Make sure `~/.cargo/bin` is in your `PATH`.

## From Source

### Prerequisites

- Rust 1.70 or later (install from https://rustup.rs)
- Cargo (comes with Rust)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/robot-accomplice/rpg.git
cd rpg

# Build in release mode
cargo build --release

# The binary will be at target/release/rpg
```

### Install Locally

```bash
# Install to ~/.cargo/bin
cargo install --path .

# Or copy manually
cp target/release/rpg ~/.cargo/bin/
```

## Verify Installation

```bash
rpg --version
```

You should see: `rpg 1.0.0`

## Troubleshooting

### Binary not found

Make sure `~/.cargo/bin` is in your PATH:

```bash
# Add to ~/.bashrc, ~/.zshrc, or equivalent
export PATH="$HOME/.cargo/bin:$PATH"
```

### Permission denied

On Unix systems, you may need to make the binary executable:

```bash
chmod +x target/release/rpg
```

