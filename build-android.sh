#!/bin/bash
set -e

echo "🐳 Building App4.Dog Android APK using Docker..."

# Create output directory
mkdir -p ./android-build-output

# Build the Docker image
echo "📦 Building Android build image..."
docker build -t app4dog-android-builder -f Dockerfile.android .

# Run the container to build APK
echo "🔨 Building APK in Docker container..."
docker run --rm \
  -v "$(pwd)/android-build-output:/app/output" \
  app4dog-android-builder

echo "✅ Android APK build complete!"
echo "📱 APK file location: ./android-build-output/"
ls -la ./android-build-output/

echo ""
echo "🚀 To install on device:"
echo "   adb install ./android-build-output/app-debug.apk"
echo ""
echo "📤 To publish to Google Play:"
echo "   Build a release APK with: docker run --rm -v \$(pwd)/android-build-output:/app/output app4dog-android-builder ./gradlew assembleRelease"