# RPG - Rust Password Generator
# Common development tasks

# Default recipe - show available commands
default:
    @just --list

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Run tests (requires cargo-nextest)
test:
    cargo nextest run

# Run tests with coverage (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --out Html --output-dir coverage

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release --verbose

# Check package for publishing
package:
    cargo package --allow-dirty

# Dry-run publish check
publish-check:
    cargo publish --dry-run

# Clean build artifacts
clean:
    cargo clean

# Run all CI checks (reproduces CI/CD pipeline)
ci-test: fmt-check lint test build-release package publish-check
    @echo "✅ All CI checks passed!"

# Run benchmarks
bench:
    cargo bench

# Generate documentation
doc:
    cargo doc --open

# Install locally
install:
    cargo install --path .

# Cross-compile for all targets (requires cross)
cross-build:
    cross build --release --target x86_64-unknown-linux-gnu
    cargo build --release --target aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
    cargo build --release --target x86_64-pc-windows-gnu
