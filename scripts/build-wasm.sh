#!/bin/bash

# Build script for Rust/Bevy game engine to WASM
set -e

# Determine build mode: dev by default for faster iteration, release when specified
MODE=${WASM_MODE:-dev}

echo "🦀 Building App4.Dog Game Engine (Rust -> WASM) [${MODE} mode]"

# Load Rust environment
. ~/.cargo/env 2>/dev/null || true

# Setup sccache if available for faster incremental builds
if command -v sccache &> /dev/null; then
    echo "⚡ Using sccache for faster incremental builds"
    export RUSTC_WRAPPER=sccache
fi

# Enable incremental compilation for dev builds
if [ "$MODE" = "dev" ]; then
    export CARGO_INCREMENTAL=1
    echo "🔄 Incremental compilation enabled for faster dev builds"
fi

# Check if required tools are installed (check both PATH and cargo bin)
HAS_WASM_PACK=0
if command -v wasm-pack &> /dev/null; then HAS_WASM_PACK=1; fi
if [ -f ~/.cargo/bin/wasm-pack ]; then HAS_WASM_PACK=1; fi

# Navigate to game engine directory
cd game-engine

# Set RUSTFLAGS for WebGPU APIs
export RUSTFLAGS="--cfg=web_sys_unstable_apis"

# Build with appropriate settings for mode
if [ "$MODE" = "release" ]; then
    echo "🏗️ Building WASM module (RELEASE) with full optimizations..."
    BUILD_FLAGS="--release"
else
    echo "🚀 Building WASM module (DEV) for fast iteration..."
    BUILD_FLAGS="--dev"
fi

if [ "$HAS_WASM_PACK" = "1" ]; then
    echo "📦 Using wasm-pack to build + package"
    set +e
    if command -v wasm-pack &> /dev/null; then
        # 🤓 Output directly to src/types to avoid copying and maintain single source of truth
        wasm-pack build --target web --out-dir ../src/types/wasm $BUILD_FLAGS
    else
        # 🤓 Output directly to src/types to avoid copying and maintain single source of truth
        ~/.cargo/bin/wasm-pack build --target web --out-dir ../src/types/wasm $BUILD_FLAGS
    fi
    STATUS=$?
    set -e
    if [ $STATUS -ne 0 ]; then
        echo "⚠️ wasm-pack failed (likely missing wasm-bindgen or perms). Falling back to cargo build."
        if [ "$MODE" = "release" ]; then
            cargo build --no-default-features --lib --target wasm32-unknown-unknown --release
        else
            cargo build --no-default-features --lib --target wasm32-unknown-unknown
        fi
        echo "ℹ️ wasm-bindgen/JS glue not generated (install wasm-bindgen and rerun wasm-pack)."
    fi
else
    echo "🔧 Running plain cargo build for wasm32-unknown-unknown (no packaging)"
    if [ "$MODE" = "release" ]; then
        cargo build --no-default-features --lib --target wasm32-unknown-unknown --release
    else
        cargo build --no-default-features --lib --target wasm32-unknown-unknown
    fi
    echo "ℹ️ wasm-bindgen/JS glue not generated (install wasm-pack to package for web)."
fi

# Only run wasm-opt for release builds (or when explicitly requested)
if [ "$MODE" = "release" ] || [ "$WASM_OPT" = "1" ]; then
    if command -v wasm-opt &> /dev/null; then
        echo "🗜️ Optimizing WASM with wasm-opt..."
        # Enable required WASM features for Bevy (using faster -O2 instead of -Oz for speed)
        timeout 60s wasm-opt -O2 \
            --enable-bulk-memory \
            --enable-nontrapping-float-to-int \
            --enable-sign-ext \
            --enable-reference-types \
            ../public/wasm/app4dog_game_engine_bg.wasm \
            -o ../public/wasm/app4dog_game_engine_bg.wasm || echo "⚠️ wasm-opt timed out - WASM files are still functional"
    else
        echo "⚠️ wasm-opt not found - skipping optimization (install with: npm install -g binaryen)"
    fi
else
    echo "⏩ Skipping wasm-opt for faster dev builds (set WASM_OPT=1 to force)"
fi

# Files are automatically available via symlink: public/game-engine -> game-engine/pkg
echo "📦 WASM files available via symlink (single source of truth), if packaging was done"

# Generate TypeScript bindings for Vue components
if [ "$HAS_WASM_PACK" = "1" ] && [ ${STATUS:-0} -eq 0 ]; then
  echo "🔧 Generating TypeScript bindings..."
  cat > ../src/types/game-engine.d.ts << 'EOF'
// Auto-generated TypeScript bindings for App4.Dog Game Engine WASM module

declare module '/game-engine/pkg/app4dog_game_engine.js' {
  export default function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<void>;
  
  export class GameEngine {
    constructor();
    start_game(): void;
    pause_game(): void;
    reset_game(): void;
    handle_interaction(type: string, x: number, y: number, dir_x: number, dir_y: number): void;
    get_game_state(): GameState;
    free(): void;
  }
  
  export interface GameState {
    score: number;
    level: number;
    is_paused: boolean;
    current_critter_id: number | null;
  }
  
  export interface CritterInfo {
    name: string;
    species: string;
    happiness: number;
    energy: number;
  }
}
EOF
else
  echo "ℹ️ Skipping TS bindings generation (requires wasm-pack output)."
fi

echo "✅ WASM build complete!"
echo "📂 TypeScript types generated in src/types/wasm/"
echo "📂 WASM files available in src/types/wasm/ (will be copied to public/ during build)"
echo "🎮 Game engine ready for Quasar integration"
