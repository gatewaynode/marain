#!/bin/bash

echo "=== Clean Rebuild Script ==="
echo "This will drop the database and rebuild everything from scratch"
echo ""

# Stop any running instances
echo "1. Stopping any running Tauri instances..."
pkill -f "cargo.*run" 2>/dev/null || true
pkill -f "target/debug/app" 2>/dev/null || true
sleep 2

# Remove the database
echo "2. Removing existing database..."
rm -f data/content/marain.db
rm -f data/content/marain.db-shm
rm -f data/content/marain.db-wal
echo "   Database removed."

# Clean build artifacts
echo "3. Cleaning build artifacts..."
cd src-tauri
cargo clean
cd ..

# Rebuild the application
echo "4. Building the application..."
cd src-tauri
cargo build
cd ..

# Start the application
echo "5. Starting the application (this will create fresh tables with rid column)..."
cd src-tauri
cargo run &
APP_PID=$!
cd ..

# Wait for the app to start and create tables
echo "6. Waiting for application to initialize..."
sleep 5

# Check if tables were created with rid column
echo "7. Verifying tables have rid column..."
echo ""
echo "=== Checking content_snippet table structure ==="
sqlite3 data/content/marain.db ".schema content_snippet" | grep -E "(rid|CREATE)" || echo "Table not found"

echo ""
echo "=== Checking content_multi table structure ==="
sqlite3 data/content/marain.db ".schema content_multi" | grep -E "(rid|CREATE)" || echo "Table not found"

echo ""
echo "=== Checking content_all_fields table structure ==="
sqlite3 data/content/marain.db ".schema content_all_fields" | grep -E "(rid|CREATE)" || echo "Table not found"

echo ""
echo "=== Application is running with PID: $APP_PID ==="
echo "The database has been rebuilt with rid columns in all tables."
echo ""
echo "You can now run the revision tests:"
echo "  ./test-revisions-detailed.sh"
echo ""
echo "To stop the application, run: kill $APP_PID"