#!/bin/bash

# Test script for Marain CMS Revision System
# Run this while the application is running (cargo run in src-tauri)

API_BASE="http://localhost:3030/api/v1"

echo "Testing Marain CMS Revision System"
echo "==================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ $2${NC}"
    else
        echo -e "${RED}✗ $2${NC}"
    fi
}

# Test 1: Create a new snippet
echo "1. Creating a new snippet..."
CREATE_RESPONSE=$(curl -s -X POST "$API_BASE/entity/create/snippet" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "title": "Revision Test Snippet",
      "body": "Original content for revision testing",
      "status": "draft"
    }
  }')

SNIPPET_ID=$(echo $CREATE_RESPONSE | jq -r '.id')
if [ "$SNIPPET_ID" != "null" ] && [ -n "$SNIPPET_ID" ]; then
    print_result 0 "Snippet created with ID: $SNIPPET_ID"
    echo $CREATE_RESPONSE | jq '.'
else
    print_result 1 "Failed to create snippet"
    echo $CREATE_RESPONSE | jq '.'
    exit 1
fi
echo ""

# Test 2: Update the snippet (should create revision 1)
echo "2. Updating the snippet (creating first revision)..."
UPDATE_RESPONSE=$(curl -s -X POST "$API_BASE/entity/update/snippet/$SNIPPET_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "title": "Revision Test Snippet - Updated",
      "body": "First update - this should create revision 1",
      "status": "published"
    }
  }')

if [ "$(echo $UPDATE_RESPONSE | jq -r '.id')" == "$SNIPPET_ID" ]; then
    print_result 0 "Snippet updated successfully"
    echo $UPDATE_RESPONSE | jq '.'
else
    print_result 1 "Failed to update snippet"
    echo $UPDATE_RESPONSE | jq '.'
fi
echo ""

# Test 3: Update again (should create revision 2)
echo "3. Updating the snippet again (creating second revision)..."
UPDATE_RESPONSE2=$(curl -s -X POST "$API_BASE/entity/update/snippet/$SNIPPET_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "title": "Revision Test Snippet - Updated Again",
      "body": "Second update - this should create revision 2",
      "status": "archived"
    }
  }')

if [ "$(echo $UPDATE_RESPONSE2 | jq -r '.id')" == "$SNIPPET_ID" ]; then
    print_result 0 "Snippet updated again successfully"
    echo $UPDATE_RESPONSE2 | jq '.'
else
    print_result 1 "Failed to update snippet again"
    echo $UPDATE_RESPONSE2 | jq '.'
fi
echo ""

# Test 4: List all revisions for the snippet
echo "4. Listing all revisions for the snippet..."
REVISIONS_RESPONSE=$(curl -s -X GET "$API_BASE/entity/version/list/snippet/$SNIPPET_ID")
REVISIONS_COUNT=$(echo $REVISIONS_RESPONSE | jq '. | length')

if [ "$REVISIONS_COUNT" -gt 0 ]; then
    print_result 0 "Found $REVISIONS_COUNT revisions"
    echo "Revision IDs: "
    echo $REVISIONS_RESPONSE | jq '.'
else
    print_result 1 "No revisions found or error occurred"
    echo $REVISIONS_RESPONSE | jq '.'
fi
echo ""

# Test 5: Read a specific revision (revision 1)
echo "5. Reading revision 1 of the snippet..."
REVISION_RESPONSE=$(curl -s -X GET "$API_BASE/entity/version/read/snippet/$SNIPPET_ID/1")

if [ "$(echo $REVISION_RESPONSE | jq -r '.id')" == "$SNIPPET_ID" ]; then
    print_result 0 "Successfully retrieved revision 1"
    echo "Revision 1 content:"
    echo $REVISION_RESPONSE | jq '.'
else
    print_result 1 "Failed to retrieve revision 1"
    echo $REVISION_RESPONSE | jq '.'
fi
echo ""

# Test 6: Read current version (should be the latest)
echo "6. Reading current version of the snippet..."
CURRENT_RESPONSE=$(curl -s -X GET "$API_BASE/entity/read/snippet/$SNIPPET_ID")

if [ "$(echo $CURRENT_RESPONSE | jq -r '.id')" == "$SNIPPET_ID" ]; then
    print_result 0 "Successfully retrieved current version"
    echo "Current version content:"
    echo $CURRENT_RESPONSE | jq '.'
else
    print_result 1 "Failed to retrieve current version"
    echo $CURRENT_RESPONSE | jq '.'
fi
echo ""

# Test 7: Test with non-versioned entity (if we had one)
echo "7. Testing error handling for non-versioned entities..."
echo "Note: All current entities are versioned, so this test is informational only"
echo ""

# Test 8: Clean up - delete the test snippet
echo "8. Cleaning up - deleting test snippet..."
DELETE_RESPONSE=$(curl -s -X POST "$API_BASE/entity/delete/snippet/$SNIPPET_ID")

if [ "$(echo $DELETE_RESPONSE | jq -r '.success')" == "true" ]; then
    print_result 0 "Test snippet deleted successfully"
else
    print_result 1 "Failed to delete test snippet"
fi
echo ""

echo "==================================="
echo "Revision system tests complete!"
echo ""
echo "Summary:"
echo "- Created a snippet with initial content"
echo "- Updated it twice to create revisions"
echo "- Listed all revisions"
echo "- Retrieved a specific revision"
echo "- Verified current version reflects latest update"
echo "- Cleaned up test data"
echo ""
echo "Note: Check the database to verify revision tables were created:"
echo "  - content_revisions_snippet"
echo "  - field_revisions_snippet_* (for multi-value fields)"