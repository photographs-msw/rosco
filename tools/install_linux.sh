#!/bin/bash

# Install liblo tools on Linux using apt
# This provides oscsend and oscrecv commands for sending/receiving OSC messages

echo "Installing liblo tools on Linux..."

# Check if running as root or with sudo
if [ "$EUID" -ne 0 ]; then
    echo "This script requires sudo privileges to install packages."
    echo "Please run: sudo $0"
    exit 1
fi

# Update package list
echo "Updating package list..."
apt update

# Install liblo-tools
echo "Installing liblo-tools..."
apt install -y liblo-tools

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