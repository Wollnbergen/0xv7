#!/bin/bash
# Build Sultan Mobile Validator for Android

set -e

echo "🏗️  Building Sultan Mobile Validator for Android..."

# Configuration
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$HOME/Android/Sdk/ndk/25.1.8937393}"
export CARGO_TARGET_DIR="/tmp/cargo-target-mobile"
export BUILD_DIR="$(pwd)/mobile-validator/android/app/src/main"

# Check dependencies
command -v cargo >/dev/null 2>&1 || { echo "❌ Cargo not found"; exit 1; }
command -v rustup >/dev/null 2>&1 || { echo "❌ Rustup not found"; exit 1; }

# Add Android targets
echo "📦 Adding Android targets..."
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# Install cargo-ndk if not present
if ! command -v cargo-ndk &> /dev/null; then
    echo "📦 Installing cargo-ndk..."
    cargo install cargo-ndk
fi

# Create JNI directory
mkdir -p "$BUILD_DIR/jniLibs/arm64-v8a"
mkdir -p "$BUILD_DIR/jniLibs/armeabi-v7a"
mkdir -p "$BUILD_DIR/jniLibs/x86"
mkdir -p "$BUILD_DIR/jniLibs/x86_64"

# Build for Android architectures
echo "🔨 Building for arm64-v8a..."
cargo ndk --target aarch64-linux-android --platform 26 -- build --release -p sultan-core

echo "🔨 Building for armeabi-v7a..."
cargo ndk --target armv7-linux-androideabi --platform 26 -- build --release -p sultan-core

echo "🔨 Building for x86_64..."
cargo ndk --target x86_64-linux-android --platform 26 -- build --release -p sultan-core

echo "🔨 Building for x86..."
cargo ndk --target i686-linux-android --platform 26 -- build --release -p sultan-core

# Copy libraries
echo "📋 Copying native libraries..."
cp "$CARGO_TARGET_DIR/aarch64-linux-android/release/libsultan_core.so" "$BUILD_DIR/jniLibs/arm64-v8a/"
cp "$CARGO_TARGET_DIR/armv7-linux-androideabi/release/libsultan_core.so" "$BUILD_DIR/jniLibs/armeabi-v7a/"
cp "$CARGO_TARGET_DIR/x86_64-linux-android/release/libsultan_core.so" "$BUILD_DIR/jniLibs/x86_64/"
cp "$CARGO_TARGET_DIR/i686-linux-android/release/libsultan_core.so" "$BUILD_DIR/jniLibs/x86/"

# Build APK
echo "📱 Building APK..."
cd mobile-validator/android
./gradlew assembleRelease

echo "✅ Android build complete!"
echo "📦 APK: mobile-validator/android/app/build/outputs/apk/release/app-release.apk"
