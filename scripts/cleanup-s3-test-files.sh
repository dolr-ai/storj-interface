#!/bin/bash

# Cleanup script for test files in Hetzner S3
# This script removes test files created during e2e tests

set -e

# Configuration
S3_ENDPOINT="${HETZNER_S3_ENDPOINT:-https://hel1.your-objectstorage.com}"
S3_BUCKET="${HETZNER_S3_BUCKET:-yral-sfw}"
S3_ACCESS_KEY="${HETZNER_S3_ACCESS_KEY}"
S3_SECRET_KEY="${HETZNER_S3_SECRET_KEY}"

# Test file patterns
TEST_VIDEO_ID="7ec40a0b9aba4307a97e8666822ed563"
HLS_TEST_VIDEO_ID="hls_test_7ec40a0b9aba4307a97e8666822ed563"
TEST_PUBLISHERS=(
    "testuser-352a268a9396c91bc8444895d9a99ae3314edea2"
    "testuser-d57eabb4c9ba464d1128a2c33199363b02f56a98"
)

# Create temporary rclone config
RCLONE_CONFIG=$(mktemp)
cat > "$RCLONE_CONFIG" << EOF
[hetzner-s3]
type = s3
provider = Other
access_key_id = $S3_ACCESS_KEY
secret_access_key = $S3_SECRET_KEY
endpoint = $S3_ENDPOINT
region = hel1
EOF

echo "Cleaning up test files from Hetzner S3..."

# Clean up regular test videos
for publisher in "${TEST_PUBLISHERS[@]}"; do
    echo "Checking for test videos under $publisher..."
    
    # List and delete test video if exists
    if rclone ls "hetzner-s3:$S3_BUCKET/$publisher/$TEST_VIDEO_ID.mp4" --config "$RCLONE_CONFIG" 2>/dev/null; then
        echo "  Deleting $publisher/$TEST_VIDEO_ID.mp4"
        rclone delete "hetzner-s3:$S3_BUCKET/$publisher/$TEST_VIDEO_ID.mp4" --config "$RCLONE_CONFIG"
    fi
done

# Clean up HLS test files
echo "Checking for HLS test files..."
if rclone ls "hetzner-s3:$S3_BUCKET/$HLS_TEST_VIDEO_ID/" --config "$RCLONE_CONFIG" 2>/dev/null | grep -q .; then
    echo "  Deleting HLS files under $HLS_TEST_VIDEO_ID/"
    rclone delete "hetzner-s3:$S3_BUCKET/$HLS_TEST_VIDEO_ID/" --config "$RCLONE_CONFIG" --rmdirs
fi

# List any remaining test files (for debugging)
echo ""
echo "Checking for any remaining test files..."
for publisher in "${TEST_PUBLISHERS[@]}"; do
    remaining=$(rclone ls "hetzner-s3:$S3_BUCKET/$publisher/" --config "$RCLONE_CONFIG" 2>/dev/null || true)
    if [ -n "$remaining" ]; then
        echo "Warning: Found remaining files under $publisher:"
        echo "$remaining"
    fi
done

# Clean up temp config
rm -f "$RCLONE_CONFIG"

echo "Cleanup complete!"