#!/bin/bash

# Deploy script for App4.Dog Game to Cloudflare Pages
# This script builds the WASM game engine and Vue app, then deploys to Cloudflare Pages

set -e

echo "🚀 Starting App4.Dog Game deployment..."

# Step 1: Install dependencies
echo "📦 Installing dependencies..."
pnpm install

# Step 2: Install just command runner
echo "🔧 Installing just..."
pnpm install just

# Step 3: Build WASM game engine
echo "🦀 Building Rust game engine to WASM..."
pnpm exec just build-wasm

# Step 4: Build Vue/Quasar application
echo "🏗️ Building Quasar application..."
pnpm run build

# Step 5: Deploy to Cloudflare Pages
echo "☁️ Deploying to Cloudflare Pages..."
npx wrangler pages deploy dist/spa

echo "✅ Deployment complete!"