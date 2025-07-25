#!/bin/bash

echo "Building Wavelog Gate with icons..."

# 检测操作系统
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Building for macOS..."
    cargo build --release
    echo "Creating macOS app bundle with icon..."
    cargo bundle --release
    echo "✅ macOS app created at: target/release/bundle/osx/Wavelog Gate.app"
    echo "   The app includes the icon.icns file"
    
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    echo "Building for Windows..."
    cargo build --release
    echo "✅ Windows executable created at: target/release/rs-wavelog-gate.exe"
    echo "   The executable includes the icon.ico file embedded as a resource"
    
else
    echo "Building for Linux..."
    cargo build --release
    echo "✅ Linux executable created at: target/release/rs-wavelog-gate"
    echo "   Note: Linux doesn't embed icons in executables. Use icon.png for desktop entries."
fi

echo "Build complete!"
