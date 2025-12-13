#!/bin/bash
# Build Sultan Mobile Validator for iOS

set -e

echo "🏗️  Building Sultan Mobile Validator for iOS..."

# Configuration
export CARGO_TARGET_DIR="/tmp/cargo-target-mobile"
export BUILD_DIR="$(pwd)/mobile-validator/ios/SultanValidator"

# Check dependencies
command -v cargo >/dev/null 2>&1 || { echo "❌ Cargo not found"; exit 1; }
command -v rustup >/dev/null 2>&1 || { echo "❌ Rustup not found"; exit 1; }
command -v xcodebuild >/dev/null 2>&1 || { echo "❌ Xcode not found"; exit 1; }

# Add iOS targets
echo "📦 Adding iOS targets..."
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# Install cargo-lipo if not present
if ! command -v cargo-lipo &> /dev/null; then
    echo "📦 Installing cargo-lipo..."
    cargo install cargo-lipo
fi

# Build universal library
echo "🔨 Building universal iOS library..."
cargo lipo --release -p sultan-core

# Create framework directory
mkdir -p "$BUILD_DIR/Frameworks"

# Copy library
echo "📋 Copying native library..."
cp "$CARGO_TARGET_DIR/universal/release/libsultan_core.a" "$BUILD_DIR/Frameworks/"

# Generate header
echo "📝 Generating header..."
cbindgen sultan-core/src/lib.rs -l c > "$BUILD_DIR/Frameworks/sultan_core.h"

# Build Xcode project
echo "📱 Building iOS app..."
cd mobile-validator/ios
xcodebuild -project SultanValidator.xcodeproj \
    -scheme SultanValidator \
    -configuration Release \
    -sdk iphoneos \
    -archivePath "$BUILD_DIR/build/SultanValidator.xcarchive" \
    archive

# Export IPA
echo "📦 Exporting IPA..."
xcodebuild -exportArchive \
    -archivePath "$BUILD_DIR/build/SultanValidator.xcarchive" \
    -exportPath "$BUILD_DIR/build" \
    -exportOptionsPlist ExportOptions.plist

echo "✅ iOS build complete!"
echo "📦 IPA: mobile-validator/ios/SultanValidator/build/SultanValidator.ipa"
