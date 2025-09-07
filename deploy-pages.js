#!/usr/bin/env node

// Custom deploy script for Cloudflare Pages
// This script builds the project and deploys it using the correct wrangler command

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🚀 Starting Cloudflare Pages deployment...');

try {
  // Check if build output exists
  const distDir = path.join(__dirname, 'dist/spa');
  
  if (!fs.existsSync(distDir)) {
    console.log('📦 Build output not found. Running build process...');
    
    // Install dependencies and build
    console.log('📦 Installing dependencies...');
    execSync('pnpm install', { stdio: 'inherit' });
    
    console.log('🔧 Installing just...');
    execSync('pnpm install just', { stdio: 'inherit' });
    
    console.log('🦀 Building WASM...');
    execSync('pnpm exec just build-wasm', { stdio: 'inherit' });
    
    console.log('🏗️ Building app...');
    execSync('pnpm run build', { stdio: 'inherit' });
  }
  
  // Deploy using wrangler pages
  console.log('☁️ Deploying to Cloudflare Pages...');
  execSync('npx wrangler pages deploy dist/spa', { stdio: 'inherit' });
  
  console.log('✅ Deployment successful!');
  
} catch (error) {
  console.error('❌ Deployment failed:', error.message);
  process.exit(1);
}