# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

App4.Dog Game is an interactive pet training application built for real dogs to play on tablets and smartphones. The game features anthropomorphic critters that pets interact with through touch, teaching vocabulary, obedience, and providing entertainment.

**Key Concepts:**
- **Pet** = Real dog/animal player 🐶
- **Critter** = Anthropomorphic game character (bird, bunny, etc.) 🐦🐰
- **Interaction** = Pet taps, swipes, or holds screen to play with critters

## Architecture

### Technology Stack
- **Frontend**: Quasar CLI (Vue 3 + TypeScript + Vite) + Capacitor for mobile
- **Game Engine**: Rust/Bevy compiled to WebAssembly (WASM)
- **Mobile**: Android/iOS deployment via Capacitor
- **Build System**: Just command runner + custom scripts

### Hybrid Architecture
This is a hybrid web/native application combining:
1. **Vue/Quasar SPA** - UI, settings, menus
2. **Rust/Bevy WASM** - High-performance game logic and rendering
3. **Capacitor** - Native mobile app deployment

## Development Commands

### Core Development
- `just dev` - Start Quasar development server (https://play.app4.dog:9000)
- `just build` - Build for production
- `just build-wasm` - Compile Rust game engine to WASM
- `just rebuild-wasm` - Clean and rebuild WASM (use when Rust changes)

### Mobile Development
- `just dev-android` - Build WASM + sync + run on Android
- `just build-android-docker` - Build Android APK using Docker (no local Android SDK)
- `just install-apk` - Install built APK to connected Android device via ADB
- `just deploy-android` - Full build + install + launch workflow
- `just cap-sync` - Sync web assets to mobile platforms
- `just cap-open-android` - Open Android Studio

### Code Quality
- `pnpm run lint` / `just lint` - ESLint with flat config
- `pnpm run format` / `just format` - Prettier formatting
- `pnpm run test` / `just test` - Run test suites

### Testing
- Main tests in `tests/` directory with dedicated package.json  
- Test files: `critter-integration.test.js`, `grid-coordinates.test.js`, `critter-id-mapping.test.js`
- Run with: `cd tests && npm install && npm test`
- All tests verify the dynamic, extensible critter system (no hardcoded IDs)

## Key Architecture Details

### WASM Integration
The game uses a sophisticated WASM build system:

1. **Build Process**: `./scripts/build-wasm.sh` compiles Rust → WASM
2. **Output**: Files generated in `public/wasm/` 
3. **TypeScript Bindings**: Auto-generated in `src/types/game-engine.d.ts`
4. **Integration**: Vue components import and instantiate the WASM module

### Game Engine (Rust/Bevy)
Located in `game-engine/` directory:
- **ECS Architecture**: Entities, Components, Systems
- **Features**: Full Bevy with WebGL2, audio, sprites, animation
- **WASM Target**: `wasm32-unknown-unknown`
- **Dependencies**: Uses custom `critter-keeper` crate

### Frontend (Vue/Quasar)
- **Configuration**: `quasar.config.ts` with HTTPS dev server
- **Boot Files**: `src/boot/` for axios, i18n setup
- **Components**: `src/components/` with game canvas and UI
- **Pages**: `src/pages/` with main game interface
- **SSL**: Development uses mkcert certs for `play.app4.dog` domain

### Mobile (Capacitor)
- **Config**: `capacitor.config.ts` and `src-capacitor/`
- **Android**: Native Android project in `android/` directory
- **Docker Build**: `./build-android.sh` for APK generation without local Android SDK

## Development Workflow

### WASM Development Cycle
When modifying Rust code:
```bash
just rebuild-wasm  # Clean + rebuild WASM
# Refresh browser to see changes
```

### 🤓 TypeScript Type System (Auto-Generated)
**IMPORTANT**: This project uses a fully auto-generated TypeScript type system to eliminate manual type maintenance.

**How it works:**
- Rust functions with `#[wasm_bindgen]` auto-generate TypeScript definitions
- Build outputs directly to `src/types/wasm/app4dog_game_engine.d.ts`
- Components import via clean re-export: `import type { GameEngine } from '../types/wasm-types'`

**When adding new WASM functions:**
1. Add `#[wasm_bindgen]` to your Rust function
2. Run `just build-wasm` 
3. TypeScript types are automatically available - no manual updates needed!

**Do NOT:**
- Manually edit files in `src/types/wasm/` (auto-generated, will be overwritten)
- Create manual type definitions that duplicate WASM exports
- Use `--no-typescript` flag in wasm-pack (breaks auto-generation)

**Architecture:**
- **Source of Truth**: Rust code with `#[wasm_bindgen]` exports
- **Auto-Generated**: `src/types/wasm/app4dog_game_engine.d.ts`
- **Clean Imports**: `src/types/wasm-types.ts` re-exports for easy importing
- **ESLint**: Auto-generated files excluded from linting to prevent noise

### Full Development Setup
```bash
just install       # Install all dependencies (Node.js + Rust tools)
just dev-full      # Clean install + build WASM + start dev server
just clean         # Clean build artifacts
just clean-all     # Clean all dependencies and artifacts
```

### Mobile Development
```bash
just dev-android    # Build + sync + run on Android device/emulator
just adb-devices    # Check connected Android devices
```

## Project Structure

```
├── src/                    # Vue/Quasar frontend
│   ├── components/         # Game UI components  
│   ├── pages/             # Game pages
│   ├── boot/              # Quasar boot files
│   └── types/             # TypeScript definitions
├── game-engine/           # Rust/Bevy WASM game engine
│   ├── src/               # Rust source code
│   └── Cargo.toml         # Rust dependencies
├── public/
│   ├── assets/            # Game assets (sprites, audio)
│   └── wasm/              # Generated WASM files
├── tests/                 # Jest test suites (separate package)
├── scripts/               # Build scripts
│   └── build-wasm.sh      # WASM build script
├── android/               # Android Capacitor project
├── src-capacitor/         # Capacitor configuration
└── justfile               # Development commands
```

## Important Configuration

### Development Server
- **URL**: https://play.app4.dog:9000 (HTTPS required for WASM features)
- **Certificates**: Located in `certs/` directory
- **CORS**: Configured for cross-origin WASM loading

### Build Optimizations
- **WASM**: Release builds use `opt-level = "z"` for size optimization
- **Post-processing**: `wasm-opt` with bulk memory and reference types enabled
- **TypeScript**: Strict mode enabled with Vue shim

## Asset Management

### Audio Files
- Located in `public/assets/audio/`
- Supports multiple formats (MP3, OGG)
- Dynamic loading with fallback logic for CORS compatibility

### Sprite Animations
- Grid-based sprite sheets (not horizontal strips)
- Dynamic coordinate calculation for critter animations
- Extensible system - new critters work without code changes

## Testing Strategy

The project includes comprehensive tests for:
- Dynamic critter ID mapping (hash-based, consistent)
- Grid coordinate generation for sprite animations
- Integration workflow (Vue → WASM → Game spawning)

All tests verify the dynamic, non-hardcoded architecture that allows adding new critters without code changes.

## Deployment

### Local Development
- `just dev` for web development
- `just dev-android` for mobile testing

### Production Builds
- `just ci-build` for CI/CD environments
- `just deploy` for Cloudflare Workers deployment (requires `wrangler.toml`)

### Mobile Distribution
- Android APK via `just build-android-docker`
- iOS builds via standard Capacitor workflow