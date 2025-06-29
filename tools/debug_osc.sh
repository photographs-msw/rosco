#!/bin/bash

# Debug script to test OSC message parsing

echo "=== OSC Message Debug Test ==="

# Check if liblo is installed
if ! command -v oscsend &> /dev/null; then
    echo "Error: oscsend not found. Please install liblo first:"
    echo "  ./tools/install_mac.sh"
    exit 1
fi

echo "1. Testing with a simple OSC message..."
echo "   oscsend localhost 8000 /test f 1.0"

# Start the server in background
echo "2. Starting server..."
cargo run --bin osc &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Send test message
echo "3. Sending test message..."
oscsend localhost 8000 /test f 1.0

# Wait a moment for processing
sleep 1

# Stop server
echo "4. Stopping server..."
kill $SERVER_PID 2>/dev/null

echo "=== Debug complete ===" 