# Installation Guide

## From Crates.io (Recommended)

Once published, install with:

```bash
cargo install rpg-cli
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

### Install Man Page (Optional)

For Unix/Linux systems, you can install the man page:

```bash
# Install to system man directory (requires sudo)
sudo cp rpg.1 /usr/local/share/man/man1/
sudo mandb  # Update man database

# Or install to user directory
mkdir -p ~/.local/share/man/man1
cp rpg.1 ~/.local/share/man/man1/
```

After installation, you can view the manual with:
```bash
man rpg
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

