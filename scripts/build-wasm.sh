#!/bin/bash

# Build script for Rust/Bevy game engine to WASM
set -e

echo "🦀 Building App4.Dog Game Engine (Rust -> WASM)"

# Check if required tools are installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Navigate to game engine directory
cd game-engine

# Build for web target
echo "🏗️ Building WASM module..."
wasm-pack build --target web --out-dir pkg --dev

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