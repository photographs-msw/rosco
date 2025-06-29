#!/bin/bash

# Install liblo tools on macOS using Homebrew
# This provides oscsend and oscrecv commands for sending/receiving OSC messages

echo "Installing liblo tools on macOS..."

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "Error: Homebrew is not installed. Please install Homebrew first:"
    echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    exit 1
fi

# Install liblo
echo "Installing liblo..."
HOMEBREW_NO_AUTO_UPDATE=1 brew install liblo

# Verify installation
if command -v oscsend &> /dev/null && command -v oscrecv &> /dev/null; then
    echo "✅ liblo tools installed successfully!"
    echo ""
    echo "Available commands:"
    echo "  oscsend - Send OSC messages"
    echo "  oscrecv - Receive OSC messages"
    echo ""
    echo "Example usage:"
    echo "  oscsend localhost 8000 /note/oscillator f 440.0 f 0.5 f 0.0 f 1000.0 s \"sine\""
    echo "  oscrecv 8000"
else
    echo "❌ Installation failed. Please check the output above for errors."
    exit 1
fi 