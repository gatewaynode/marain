#!/bin/bash

echo "Rebuilding and testing revision system..."
echo "========================================="
echo ""

# Kill any existing cargo run processes
echo "Stopping any existing processes..."
pkill -f "cargo run" 2>/dev/null
sleep 2

# Rebuild
echo "Building the application..."
cd src-tauri && cargo build

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

# Start the application in background
echo "Starting the application..."
cargo run > /tmp/marain.log 2>&1 &
APP_PID=$!

# Wait for the application to start
echo "Waiting for application to start..."
for i in {1..10}; do
    if curl -s http://localhost:3030/api/v1/health > /dev/null 2>&1; then
        echo "Application is ready!"
        break
    fi
    echo "Waiting... ($i/10)"
    sleep 2
done

# Go back to project root
cd ..

# Run the detailed test
echo ""
echo "Running revision system tests..."
echo "================================"
./test-revisions-detailed.sh

# Optional: Kill the application after testing
# kill $APP_PID 2>/dev/null

echo ""
echo "Test complete. Application is still running in background."
echo "To stop it, run: pkill -f 'cargo run'"