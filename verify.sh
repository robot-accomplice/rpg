#!/bin/bash
# Verification script for RPG before publishing

set -e

echo "🔍 Verifying RPG for publication..."
echo ""

echo "📦 Checking package..."
cargo package --list --allow-dirty > /dev/null 2>&1 || cargo package --list > /dev/null 2>&1
echo "✅ Package structure valid"

echo ""
echo "🧪 Running tests..."
cargo test --all --quiet
echo "✅ All tests pass"

echo ""
echo "🔍 Running clippy..."
cargo clippy -- -D warnings
echo "✅ Clippy passes"

echo ""
echo "📝 Checking formatting..."
cargo fmt -- --check
echo "✅ Code is formatted"

echo ""
echo "📚 Building documentation..."
cargo doc --no-deps --quiet
echo "✅ Documentation builds"

echo ""
echo "📦 Testing package creation..."
cargo package --allow-dirty > /dev/null 2>&1 || cargo package > /dev/null 2>&1
echo "✅ Package can be created"

echo ""
echo "🚀 Testing dry-run publish..."
cargo publish --dry-run --allow-dirty > /dev/null 2>&1 || cargo publish --dry-run > /dev/null 2>&1
echo "✅ Dry-run publish successful"

echo ""
echo "✨ All checks passed! Ready for publication."
echo ""
echo "Next steps:"
echo "1. cargo login <your-api-token>"
echo "2. cargo publish"
echo "3. (Optional) Set up Codecov for coverage tracking (see PUBLISHING.md)"
