# App4.Dog Game Development Commands
# Interactive pet training game built with Quasar/Capacitor and Rust/Bevy WASM

# Default recipe - show available commands
default:
    @just --list

# Development commands
dev:
    @echo "🚀 Starting Quasar development server..."
    pnpm run dev

build:
    @echo "🏗️ Building Quasar application..."
    pnpm run build

# Game engine commands
build-wasm:
    @echo "🦀 Building Rust game engine to WASM..."
    chmod +x scripts/build-wasm.sh
    ./scripts/build-wasm.sh

wasm-dev:
    @echo "🔄 Building WASM in development mode..."
    cd game-engine && wasm-pack build --target web --out-dir ../public/game-engine --dev

wasm-release:
    @echo "🚀 Building WASM in release mode..."
    cd game-engine && wasm-pack build --target web --out-dir ../public/game-engine --release

# Mobile development
dev-android: build-wasm
    @echo "📱 Starting Android development..."
    pnpm run build
    npx cap sync android
    npx cap run android

dev-ios: build-wasm
    @echo "📱 Starting iOS development..."
    pnpm run build
    npx cap sync ios
    npx cap run ios

# Capacitor mobile commands
cap-sync: build
    @echo "🔄 Syncing with Capacitor..."
    npx cap sync

cap-open-android:
    @echo "📱 Opening Android Studio..."
    npx cap open android

cap-open-ios:
    @echo "📱 Opening Xcode..."
    npx cap open ios


# Testing and linting
test:
    @echo "🧪 Running tests..."
    pnpm run test

lint:
    @echo "🔍 Running ESLint..."
    pnpm run lint

format:
    @echo "✨ Formatting code..."
    pnpm run format

# Git and deployment
commit: lint format
    @echo "📝 Staging and committing changes..."
    git add .
    git status

push: commit
    @echo "🚀 Pushing to GitHub..."
    git push origin main

# Development setup
install:
    @echo "📦 Installing dependencies..."
    pnpm install
    @echo "🦀 Installing Rust tools..."
    rustup target add wasm32-unknown-unknown
    cargo install wasm-pack
    @echo "✅ Development environment ready!"

# Clean commands
clean:
    @echo "🧹 Cleaning build artifacts..."
    rm -rf dist/
    rm -rf public/game-engine/
    rm -rf game-engine/pkg/
    rm -rf game-engine/target/

clean-all: clean
    @echo "🧹 Cleaning all dependencies..."
    rm -rf node_modules/
    rm -rf game-engine/target/

# Help and information
info:
    @echo "📋 App4.Dog Game Project Information:"
    @echo "  Frontend: Quasar (Vue 3 + TypeScript) + Capacitor"
    @echo "  Game Engine: Rust/Bevy compiled to WASM"
    @echo "  Mobile: Android/iOS via Capacitor"
    @echo "  Assets: Migrated from puppyplay-godot-droid"
    @echo ""
    @echo "🎯 Purpose: Interactive pet training game for dogs"
    @echo "🐾 Players: Real pets interact with anthropomorphic critters"
    @echo ""
    @echo "📁 Key directories:"
    @echo "  src/          - Vue/Quasar frontend"
    @echo "  game-engine/  - Rust/Bevy game logic"
    @echo "  public/assets/ - Game sprites, audio, fonts"
    @echo "  scripts/      - Build and deployment scripts"

# Development workflow
dev-full: clean install build-wasm dev

# Release workflow
release: clean install wasm-release build

# Smart deployment - detects and installs what's needed
deploy:
    @echo "🚀 Starting smart deployment..."
    @# Check and install Rust if needed
    @if ! command -v rustc &> /dev/null; then \
        echo "🦀 Installing Rust toolchain..."; \
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
        source ~/.cargo/env; \
        rustup target add wasm32-unknown-unknown; \
    else \
        echo "✅ Rust already installed"; \
        source ~/.cargo/env 2>/dev/null || true; \
    fi
    @# Check and install wasm-pack if needed
    @if ! command -v wasm-pack &> /dev/null; then \
        echo "📦 Installing wasm-pack..."; \
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh; \
    else \
        echo "✅ wasm-pack already installed"; \
    fi
    @# Install dependencies if needed
    @if [ ! -d "node_modules" ]; then \
        echo "📦 Installing dependencies..."; \
        pnpm install; \
    else \
        echo "✅ Dependencies already installed"; \
    fi
    @# Build WASM
    @echo "🦀 Building WASM..."
    @chmod +x scripts/build-wasm.sh
    @./scripts/build-wasm.sh
    @# Build app
    @echo "🏗️ Building application..."
    @pnpm run build
    @# Deploy
    @echo "☁️ Deploying to Cloudflare Worker..."
    @npx wrangler deploy
    @echo "✅ Deployment complete!"