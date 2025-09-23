#!/bin/bash
set -euo pipefail

# R2 sync script for Cloudflare deployment with rsync-like behavior
# Usage: ./scripts/r2-sync.sh <bucket-name> [source-dir]

BUCKET_NAME="${1:-}"
SOURCE_DIR="${2:-dist/spa}"

if [ -z "$BUCKET_NAME" ]; then
    echo "❌ Usage: $0 <bucket-name> [source-dir]"
    echo "Example: $0 gameplay-live dist/spa"
    exit 1
fi

if [ ! -d "$SOURCE_DIR" ]; then
    echo "❌ Source directory '$SOURCE_DIR' does not exist"
    exit 1
fi

echo "🚀 Syncing '$SOURCE_DIR' to R2 bucket '$BUCKET_NAME'..."

# Ensure required env vars for wrangler are present
if [ -z "${CLOUDFLARE_API_TOKEN:-}" ]; then
    echo "❌ CLOUDFLARE_API_TOKEN is not set. Add CF_API_TOKEN secret in CI."
    exit 1
fi
if [ -z "${CLOUDFLARE_ACCOUNT_ID:-}" ]; then
    echo "❌ CLOUDFLARE_ACCOUNT_ID is not set. Add CF_ACCOUNT_ID secret in CI."
    exit 1
fi

# Content type mappings (comprehensive list)
declare -A CONTENT_TYPES=(
    ["wasm"]="application/wasm"
    ["js"]="application/javascript"
    ["mjs"]="application/javascript"
    ["html"]="text/html"
    ["css"]="text/css"
    ["json"]="application/json"
    ["png"]="image/png"
    ["jpg"]="image/jpeg"
    ["jpeg"]="image/jpeg"
    ["gif"]="image/gif"
    ["svg"]="image/svg+xml"
    ["webp"]="image/webp"
    ["ico"]="image/x-icon"
    ["pdf"]="application/pdf"
    ["mp3"]="audio/mpeg"
    ["ogg"]="audio/ogg"
    ["wav"]="audio/wav"
    ["m4a"]="audio/mp4"
    ["woff"]="font/woff"
    ["woff2"]="font/woff2"
    ["ttf"]="font/ttf"
    ["otf"]="font/otf"
    ["eot"]="font/eot"
    ["mp4"]="video/mp4"
    ["webm"]="video/webm"
    ["xml"]="application/xml"
    ["txt"]="text/plain"
    ["md"]="text/markdown"
    ["zip"]="application/zip"
    ["gz"]="application/gzip"
    ["map"]="application/json"
)

# Try using rclone if available (much faster and has true sync)
if command -v rclone >/dev/null 2>&1 && [ -n "${CLOUDFLARE_ACCESS_KEY_ID:-}" ] && [ -n "${CLOUDFLARE_SECRET_ACCESS_KEY:-}" ]; then
    echo "📡 Using rclone for efficient sync..."

    # Check if rclone remote exists, if not create it
    if ! rclone listremotes | grep -q "^r2:"; then
        echo "🔧 Configuring rclone for Cloudflare R2..."
        rclone config create r2 s3 \
            provider=Cloudflare \
            access_key_id="$CLOUDFLARE_ACCESS_KEY_ID" \
            secret_access_key="$CLOUDFLARE_SECRET_ACCESS_KEY" \
            endpoint="https://$CLOUDFLARE_ACCOUNT_ID.r2.cloudflarestorage.com"
    fi

    # Sync with rclone (much more efficient)
    rclone sync "$SOURCE_DIR" "r2:$BUCKET_NAME" \
        --progress \
        --delete-after \
        --checksum \
        --transfers=10 \
        --checkers=20

    echo "✅ Rclone sync complete"
    exit 0
fi

# Fallback to wrangler (slower but works without additional setup)
echo "📡 Using wrangler for sync (install rclone for faster syncing)..."

# Ensure bucket exists (create if missing)
echo "🪣 Ensuring R2 bucket '$BUCKET_NAME' exists..."
set +e
bucket_exists_json=$(wrangler r2 bucket list --account-id "$CLOUDFLARE_ACCOUNT_ID" --output json 2>/dev/null)
if [ $? -ne 0 ] || ! echo "$bucket_exists_json" | jq -e '. | type == "array"' >/dev/null 2>&1; then
  # Fallback to non-JSON list parsing if JSON output unsupported
  bucket_list_text=$(wrangler r2 bucket list --account-id "$CLOUDFLARE_ACCOUNT_ID" 2>/dev/null || true)
  echo "$bucket_list_text" | grep -q "\b$BUCKET_NAME\b"
  exists=$?
else
  echo "$bucket_exists_json" | jq -r '.[].name' | grep -qx "$BUCKET_NAME"
  exists=$?
fi
set -e
if [ $exists -ne 0 ]; then
  echo "🪣 Bucket not found. Creating '$BUCKET_NAME'..."
  wrangler r2 bucket create "$BUCKET_NAME" --account-id "$CLOUDFLARE_ACCOUNT_ID"
else
  echo "✅ Bucket exists"
fi

# Change to source directory for relative paths
cd "$SOURCE_DIR"

# Get current bucket contents for cleanup
echo "🗂️  Getting current bucket contents..."
bucket_files=$(mktemp)
wrangler r2 object list "$BUCKET_NAME" --account-id "$CLOUDFLARE_ACCOUNT_ID" --output json 2>/dev/null | \
    jq -r '.objects[]?.key // empty' > "$bucket_files" || touch "$bucket_files"

# Get local files
local_files=$(mktemp)
find . -type f | sed 's|^\./||' | sort > "$local_files"

# Upload/update files with proper content types  
echo "📤 Uploading files..."
while read -r file; do
    # Skip if file is empty
    [ -n "$file" ] || continue
    
    # Get file extension
    ext="${file##*.}"
    ext_lower=$(echo "$ext" | tr '[:upper:]' '[:lower:]')
    
    # Determine content type
    content_type="${CONTENT_TYPES[$ext_lower]:-application/octet-stream}"
    
    # Upload to R2
    echo "📤 $file (${content_type})"
    wrangler r2 object put "$BUCKET_NAME/$file" \
        --file="$file" \
        --content-type="$content_type" \
        --account-id "$CLOUDFLARE_ACCOUNT_ID"
done < "$local_files"

# Clean up orphaned files (files in bucket but not in local)
echo "🧹 Cleaning up orphaned files..."
comm -23 <(sort "$bucket_files") "$local_files" | while read -r orphan; do
    [ -n "$orphan" ] || continue
    echo "🗑️  Removing: $orphan"
    wrangler r2 object delete "$BUCKET_NAME/$orphan" --account-id "$CLOUDFLARE_ACCOUNT_ID"
done

# Capture stats before cleanup
total_files=$(wc -l < "$local_files" || echo 0)
total_size=$(du -sh . | cut -f1)

# Cleanup temp files
rm -f "$bucket_files" "$local_files"

echo "✅ Sync complete to R2 bucket '$BUCKET_NAME'"
echo "📊 Total files: $total_files"
echo "📦 Total size: $total_size"
