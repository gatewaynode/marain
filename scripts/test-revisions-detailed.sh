#!/bin/bash

# Detailed test script for Marain CMS Revision System
# Run this after the application is fully started

API_BASE="http://localhost:3030/api/v1"

echo "=========================================="
echo "Marain CMS Revision System - Detailed Test"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ $2${NC}"
    else
        echo -e "${RED}✗ $2${NC}"
    fi
}

# Check if server is running
echo "Checking server health..."
HEALTH_CHECK=$(curl -s -X GET "$API_BASE/health" 2>/dev/null)
if [ -z "$HEALTH_CHECK" ]; then
    echo -e "${RED}ERROR: Server is not running or not responding${NC}"
    echo "Please start the application first with:"
    echo "  cd src-tauri && cargo run"
    exit 1
fi
echo -e "${GREEN}✓ Server is healthy${NC}"
echo ""

# Test 1: Create a new snippet
echo -e "${YELLOW}Test 1: Creating a new snippet...${NC}"
CREATE_RESPONSE=$(curl -s -X POST "$API_BASE/entity/create/snippet" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "title": "Revision Test Snippet",
      "body": "Original content for revision testing",
      "status": "draft"
    }
  }')

SNIPPET_ID=$(echo $CREATE_RESPONSE | jq -r '.id' 2>/dev/null)
if [ "$SNIPPET_ID" != "null" ] && [ -n "$SNIPPET_ID" ]; then
    print_result 0 "Snippet created with ID: $SNIPPET_ID"
    echo "Response:"
    echo $CREATE_RESPONSE | jq '.' 2>/dev/null || echo $CREATE_RESPONSE
else
    print_result 1 "Failed to create snippet"
    echo "Response:"
    echo $CREATE_RESPONSE | jq '.' 2>/dev/null || echo $CREATE_RESPONSE
    exit 1
fi
echo ""

# Wait a moment for database to settle
sleep 1

# Test 2: Update the snippet (should create revision 1)
echo -e "${YELLOW}Test 2: Updating the snippet (should create revision with rid=1)...${NC}"
UPDATE_RESPONSE=$(curl -s -X POST "$API_BASE/entity/update/snippet/$SNIPPET_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "title": "Revision Test Snippet - Updated",
      "body": "First update - this should create revision 1",
      "status": "published"
    }
  }')

UPDATE_STATUS=$(echo $UPDATE_RESPONSE | jq -r '.id' 2>/dev/null)
if [ "$UPDATE_STATUS" == "$SNIPPET_ID" ]; then
    print_result 0 "Snippet updated successfully"
    echo "Response:"
    echo $UPDATE_RESPONSE | jq '.' 2>/dev/null || echo $UPDATE_RESPONSE
else
    print_result 1 "Failed to update snippet"
    echo "Error Response:"
    echo $UPDATE_RESPONSE | jq '.' 2>/dev/null || echo $UPDATE_RESPONSE
    echo ""
    echo "This might be due to revision table creation issues."
fi
echo ""

# Wait a moment
sleep 1

# Test 3: Update again (should create revision 2)
echo -e "${YELLOW}Test 3: Updating the snippet again (should create revision with rid=2)...${NC}"
UPDATE_RESPONSE2=$(curl -s -X POST "$API_BASE/entity/update/snippet/$SNIPPET_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "title": "Revision Test Snippet - Updated Again",
      "body": "Second update - this should create revision 2",
      "status": "archived"
    }
  }')

UPDATE_STATUS2=$(echo $UPDATE_RESPONSE2 | jq -r '.id' 2>/dev/null)
if [ "$UPDATE_STATUS2" == "$SNIPPET_ID" ]; then
    print_result 0 "Snippet updated again successfully"
    echo "Response:"
    echo $UPDATE_RESPONSE2 | jq '.' 2>/dev/null || echo $UPDATE_RESPONSE2
else
    print_result 1 "Failed to update snippet again"
    echo "Error Response:"
    echo $UPDATE_RESPONSE2 | jq '.' 2>/dev/null || echo $UPDATE_RESPONSE2
fi
echo ""

# Test 4: List all revisions for the snippet
echo -e "${YELLOW}Test 4: Listing all revisions for the snippet...${NC}"
REVISIONS_RESPONSE=$(curl -s -X GET "$API_BASE/entity/version/list/snippet/$SNIPPET_ID")
REVISIONS_COUNT=$(echo $REVISIONS_RESPONSE | jq '. | length' 2>/dev/null)

if [ "$REVISIONS_COUNT" ] && [ "$REVISIONS_COUNT" -gt 0 ]; then
    print_result 0 "Found $REVISIONS_COUNT revisions"
    echo "Revision IDs: "
    echo $REVISIONS_RESPONSE | jq '.' 2>/dev/null || echo $REVISIONS_RESPONSE
else
    print_result 1 "No revisions found or error occurred"
    echo "Response:"
    echo $REVISIONS_RESPONSE | jq '.' 2>/dev/null || echo $REVISIONS_RESPONSE
fi
echo ""

# Test 5: Read a specific revision (revision 1)
echo -e "${YELLOW}Test 5: Reading revision 1 of the snippet...${NC}"
REVISION_RESPONSE=$(curl -s -X GET "$API_BASE/entity/version/read/snippet/$SNIPPET_ID/1")

REVISION_ID=$(echo $REVISION_RESPONSE | jq -r '.id' 2>/dev/null)
if [ "$REVISION_ID" == "$SNIPPET_ID" ]; then
    print_result 0 "Successfully retrieved revision 1"
    echo "Revision 1 content:"
    echo $REVISION_RESPONSE | jq '.' 2>/dev/null || echo $REVISION_RESPONSE
else
    print_result 1 "Failed to retrieve revision 1"
    echo "Response:"
    echo $REVISION_RESPONSE | jq '.' 2>/dev/null || echo $REVISION_RESPONSE
fi
echo ""

# Test 6: Read current version (should be the latest with rid=3)
echo -e "${YELLOW}Test 6: Reading current version of the snippet...${NC}"
CURRENT_RESPONSE=$(curl -s -X GET "$API_BASE/entity/read/snippet/$SNIPPET_ID")

CURRENT_ID=$(echo $CURRENT_RESPONSE | jq -r '.id' 2>/dev/null)
if [ "$CURRENT_ID" == "$SNIPPET_ID" ]; then
    print_result 0 "Successfully retrieved current version"
    echo "Current version content:"
    echo $CURRENT_RESPONSE | jq '.' 2>/dev/null || echo $CURRENT_RESPONSE
    
    # Check if the content matches the last update
    CURRENT_TITLE=$(echo $CURRENT_RESPONSE | jq -r '.data.title' 2>/dev/null)
    if [ "$CURRENT_TITLE" == "Revision Test Snippet - Updated Again" ]; then
        echo -e "${GREEN}✓ Current version contains the latest updates${NC}"
    else
        echo -e "${YELLOW}⚠ Current version may not reflect the latest update${NC}"
    fi
else
    print_result 1 "Failed to retrieve current version"
    echo "Response:"
    echo $CURRENT_RESPONSE | jq '.' 2>/dev/null || echo $CURRENT_RESPONSE
fi
echo ""

# Test 7: Test error handling for non-existent revision
echo -e "${YELLOW}Test 7: Testing error handling for non-existent revision...${NC}"
NONEXISTENT_RESPONSE=$(curl -s -X GET "$API_BASE/entity/version/read/snippet/$SNIPPET_ID/999")
ERROR_CODE=$(echo $NONEXISTENT_RESPONSE | jq -r '.error.code' 2>/dev/null)

if [ "$ERROR_CODE" == "ENTITY_NOT_FOUND" ]; then
    print_result 0 "Correctly returned error for non-existent revision"
else
    echo -e "${YELLOW}⚠ Unexpected response for non-existent revision${NC}"
    echo $NONEXISTENT_RESPONSE | jq '.' 2>/dev/null || echo $NONEXISTENT_RESPONSE
fi
echo ""

# Test 8: Clean up - delete the test snippet
echo -e "${YELLOW}Test 8: Cleaning up - deleting test snippet...${NC}"
DELETE_RESPONSE=$(curl -s -X POST "$API_BASE/entity/delete/snippet/$SNIPPET_ID")

DELETE_SUCCESS=$(echo $DELETE_RESPONSE | jq -r '.success' 2>/dev/null)
if [ "$DELETE_SUCCESS" == "true" ]; then
    print_result 0 "Test snippet deleted successfully"
else
    print_result 1 "Failed to delete test snippet"
    echo "Response:"
    echo $DELETE_RESPONSE | jq '.' 2>/dev/null || echo $DELETE_RESPONSE
fi
echo ""

echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo ""
echo "The revision system test has completed."
echo ""
echo "Expected behavior:"
echo "1. Create a snippet (rid=1 in main table)"
echo "2. First update creates a revision and increments rid to 2"
echo "3. Second update creates another revision and increments rid to 3"
echo "4. Revision table should contain the historical versions"
echo "5. Main table should contain the latest version"
echo ""
echo "Database tables to check:"
echo "  - content_snippet (main table with current data)"
echo "  - content_revisions_snippet (revision history)"
echo "  - field_revisions_snippet_* (for multi-value fields if any)"
echo ""
echo "You can verify the database state with:"
echo "  sqlite3 data/marain.db"
echo "  .tables"
echo "  SELECT * FROM content_snippet;"
echo "  SELECT * FROM content_revisions_snippet;"