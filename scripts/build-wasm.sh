#!/bin/bash

# Build script for Rust/Bevy game engine to WASM
set -e

echo "🦀 Building App4.Dog Game Engine (Rust -> WASM)"

# Load Rust environment
. ~/.cargo/env 2>/dev/null || true

# Check if required tools are installed (check both PATH and cargo bin)
if ! command -v wasm-pack &> /dev/null && ! [ -f ~/.cargo/bin/wasm-pack ]; then
    echo "❌ wasm-pack not found. Installing..."
    ~/.cargo/bin/cargo install wasm-pack || curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Navigate to game engine directory
cd game-engine

# Build for web target using absolute path if needed
echo "🏗️ Building WASM module..."
if command -v wasm-pack &> /dev/null; then
    wasm-pack build --target web --out-dir pkg --release
else
    ~/.cargo/bin/wasm-pack build --target web --out-dir pkg --release
fi

# Post-process with wasm-opt for additional size reduction
if command -v wasm-opt &> /dev/null; then
    echo "🗜️ Optimizing WASM with wasm-opt..."
    wasm-opt -Oz pkg/app4dog_game_engine_bg.wasm -o pkg/app4dog_game_engine_bg.wasm
else
    echo "⚠️ wasm-opt not found - install with: npm install -g binaryen"
fi

# Copy generated files to public directory for Quasar
echo "📦 Copying WASM files to public directory..."
mkdir -p ../public/game-engine
cp -r pkg/* ../public/game-engine/

# Generate TypeScript bindings for Vue components
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

echo "✅ WASM build complete!"
echo "📂 Files generated in public/game-engine/"
echo "🎮 Game engine ready for Quasar integration"