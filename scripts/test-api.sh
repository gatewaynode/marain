#!/bin/bash

# Test script for Marain CMS API endpoints
# Run this while the application is running (cargo run in src-tauri)

API_BASE="http://localhost:3030/api/v1"

echo "Testing Marain CMS API Endpoints"
echo "================================="
echo ""

# Test health endpoint
echo "1. Testing Health Check endpoint..."
curl -s -X GET "$API_BASE/health" | jq '.'
echo ""

# Test list snippets
echo "2. Testing List Snippets endpoint..."
curl -s -X GET "$API_BASE/entity/list/snippet" | jq '.'
echo ""

# Test read a specific snippet (using mock ID for now)
echo "3. Testing Read Single Snippet endpoint..."
curl -s -X GET "$API_BASE/entity/read/snippet/test-id" | jq '.' 2>/dev/null || echo "Entity not found (expected for mock ID)"
echo ""

# Test create a new snippet
echo "4. Testing Create Snippet endpoint..."
curl -s -X POST "$API_BASE/entity/create/snippet" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "title": "Test Snippet",
      "body": "This is a test snippet created via API",
      "status": "draft"
    }
  }' | jq '.'
echo ""

# Test Swagger UI availability
echo "5. Testing Swagger UI availability..."
curl -s -o /dev/null -w "Swagger UI HTTP Status: %{http_code}\n" "$API_BASE/swagger"
echo ""

echo "================================="
echo "API tests complete!"
echo ""
echo "Note: You can also visit the Swagger UI at: http://localhost:3030/api/v1/swagger"