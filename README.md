# App4.Dog Interactive Game 🐕🎮

An interactive pet training game built with **Quasar/Capacitor** and **Rust/Bevy WASM** - the modern successor to puppyplay-godot-droid.

## 🎯 Overview

App4.Dog Game is designed for **real pets** (especially dogs) to play on tablets and smartphones. The game features **anthropomorphic critters** that pets interact with through touch, teaching vocabulary, obedience, and providing entertainment.

### Key Concepts
- **Pet** = Real dog/animal player 🐶
- **Critter** = Anthropomorphic game character (bird, bunny, etc.) 🐦🐰
- **Interaction** = Pet taps, swipes, or holds screen to play with critters

## 🏗️ Architecture

### Frontend Stack
- **Quasar CLI** - Vue 3 + TypeScript + Vite
- **Capacitor** - Mobile app deployment (Android/iOS)
- **Material Design 3** - UI components

### Game Engine
- **Rust** - Core game logic with Bevy ECS
- **WebAssembly (WASM)** - Browser/mobile execution
- **Bevy Engine** - Entity Component System for game objects

### Mobile Deployment
- **Android** - Native APK via Capacitor
- **iOS** - Native app via Capacitor (future)

## 🚀 Quick Start

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js/pnpm
npm install -g pnpm

# Install Quasar CLI
pnpm install -g @quasar/cli
```

### Development Setup
```bash
# Clone and setup
git clone https://github.com/app4dog/app4dog-game.git
cd app4dog-game

# Install dependencies
pnpm install

# Add Rust WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack for WASM builds
cargo install wasm-pack

# Build game engine to WASM
./scripts/build-wasm.sh

# Start development server
pnpm run dev
```

Visit `http://localhost:9000` to see the game!

### WASM Build Process

The game uses a two-part build system:

1. **Rust → WASM**: `./scripts/build-wasm.sh` compiles the Bevy game engine to WebAssembly
2. **Quasar Build**: Standard Vue.js/Quasar build process with WASM integration

```bash
# Manual WASM build (done automatically by script)
cd game-engine
wasm-pack build --target web --out-dir pkg --dev
cd ..

# Files are copied to public/game-engine/ for Quasar access
# TypeScript bindings generated in src/types/game-engine.d.ts
```

## 📱 Mobile Development

### Android
```bash
# Build and run on Android device/emulator
just dev-android

# Or step by step:
just build-wasm
pnpm run build
npx cap sync android
npx cap open android
```

### iOS (Future)
```bash
# Build and run on iOS device/simulator
just dev-ios
```

## 🎮 Game Features

### Critter System
- **Chirpy** (Bird) - Flies around, responds to taps
- **Bouncy** (Bunny) - Hops and follows swipe gestures  
- **Buddy** (Dog) - High obedience, great for training
- **Whiskers** (Cat) - Independent, challenging gameplay

### Interaction Types
- **Tap** - Pet paws/nose tap triggers critter responses
- **Swipe** - Directional movement commands
- **Hold** - Keeps critters in place

### Training Modes
- **Basic Commands** - Sit, stay, come
- **Object Recognition** - Toys, food, household items
- **People Recognition** - Family members, visitors
- **Emotion Training** - Happy, sad, excited responses

## 🛠️ Development Commands

```bash
# Core development
just dev              # Start Quasar dev server
just build           # Build for production
just build-wasm      # Compile Rust to WASM

# Mobile development  
just dev-android     # Android development
just dev-ios         # iOS development (future)


# Code quality
just lint            # ESLint
just format          # Prettier
just test            # Run tests

# Deployment
just commit          # Stage and commit
just push            # Push to GitHub

# Maintenance
just clean           # Clean build files
just info            # Project information
```

## 📁 Project Structure

```
app4dog-game/
├── src/                    # Vue/Quasar frontend
│   ├── components/         # Game components
│   │   ├── GameCanvas.vue  # Main game canvas
│   │   ├── CritterSelection.vue
│   │   └── GameSettings.vue
│   ├── pages/
│   │   └── GamePage.vue    # Game page (/game)
│   └── types/              # TypeScript definitions
├── game-engine/            # Rust/Bevy game logic
│   ├── src/
│   │   ├── lib.rs          # WASM entry point
│   │   ├── game.rs         # Game plugin & state
│   │   ├── components.rs   # ECS components
│   │   ├── systems.rs      # Game systems
│   │   └── resources.rs    # Game resources
│   └── Cargo.toml          # Rust dependencies
├── public/assets/          # Game assets
│   ├── sprites/            # Critter animations
│   ├── audio/              # Sound effects
│   └── fonts/              # Custom fonts
├── scripts/                # Build scripts
└── justfile               # Development commands
```

## 🎨 Game Assets

Asset structure ready for critter animations and audio:

- `public/assets/sprites/` - Critter sprite sheets and animations
- `public/assets/audio/positive/` - Positive feedback sounds  
- `public/assets/fonts/` - Custom fonts for game UI

*Note: Actual asset files should be added to these directories*

## 🔧 Technical Details

### Rust/Bevy Game Engine
- **ECS Architecture** - Entities, Components, Systems
- **WASM Compilation** - Via wasm-pack for web deployment
- **JavaScript Bridge** - Bidirectional communication with Vue
- **Asset Loading** - Sprites, audio, fonts from public directory

### Vue/Quasar Integration
- **GameCanvas.vue** - Main game container with touch handling
- **Touch Events** - Converted to game interactions
- **State Management** - Reactive game state with Vue refs
- **Mobile Optimization** - Touch-first interface design

### Capacitor Mobile
- **Native APIs** - Device vibration, screen wake lock
- **Performance** - Hardware-accelerated rendering
- **Distribution** - Google Play Store, Apple App Store

## 🐾 Compared to puppyplay-godot-droid

### Improvements
- ✅ **Easier Testing** - Web-first development, no Android emulator required
- ✅ **Better Architecture** - Clean separation of concerns  
- ✅ **Type Safety** - Full TypeScript + Rust type safety
- ✅ **Modern Stack** - Latest Vue 3, Bevy, and mobile tooling
- ✅ **Debuggability** - Better error handling and logging

### Migration Benefits
- **Reduced Complexity** - Simpler build process
- **Faster Iteration** - Hot reload in browser
- **Better Testing** - Unit tests for both Rust and Vue code
- **Mobile-First** - Designed specifically for tablets

## 📈 Development Roadmap

### Phase 1: Core Game ✅
- [x] Project setup and architecture
- [x] Quasar + Vue 3 + TypeScript frontend
- [x] Rust/Bevy game engine with ECS
- [x] WASM compilation and integration
- [x] Touch interaction handling
- [x] Basic game state management
- [x] Build system and scripts

### Phase 2: Enhanced Gameplay 🚧
- [ ] Asset loading and rendering
- [ ] Critter animations and sprites
- [ ] Sound system integration
- [ ] Score and progression system
- [ ] Game canvas rendering

### Phase 3: Training Features 📋
- [ ] Vocabulary recognition
- [ ] Custom pet profiles
- [ ] Training progress tracking
- [ ] Achievement system

### Phase 4: Mobile Polish 📋  
- [ ] Android build optimization
- [ ] iOS support
- [ ] App store deployment
- [ ] Performance optimization

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make changes and test: `just dev`
4. Commit with good messages: `just commit`
5. Push and create a Pull Request

## 📄 License

MIT License - See [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **puppyplay-godot-droid** - Original proof of concept
- **Bevy Engine** - Rust game engine
- **Quasar Framework** - Vue.js framework
- **App4.Dog Ecosystem** - Backend infrastructure

---

**Built with ❤️ for pets and their humans**
