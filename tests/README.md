# Critter System Tests

This directory contains comprehensive tests for the App4.Dog critter system, verifying that the dynamic critter loading and sprite animation coordinates work correctly.

## Test Results ✅

All JavaScript tests are **PASSING**:

```
Test Suites: 3 passed, 3 total
Tests:       12 passed, 12 total
Time:        0.52 s
```

## Test Coverage

### 1. Dynamic Critter ID Mapping (`critter-id-mapping.test.js`)
- ✅ Generates consistent hash IDs for known critters
- ✅ Produces different IDs for different critters  
- ✅ Handles new critters dynamically (no hardcoding)
- ✅ Produces IDs in valid range (0-999)

**Key Results:**
- `chirpy_bird` → `167`
- `bouncy_bunny` → `307`
- Future critters like `sleepy_cat` → `65`, `wise_owl` → `873` work automatically

### 2. Grid Coordinate Generation (`grid-coordinates.test.js`)
- ✅ Generates correct coordinates for chirpy_bird (3x2 Grid)
- ✅ Generates correct coordinates for bouncy_bunny (4x4 Grid with frames [0,14])
- ✅ Handles edge cases gracefully
- ✅ Verifies Grid coordinate mathematical formula

**Key Results:**
- Chirpy bird (3x2 grid): `(0,0), (1000,0), (2000,0), (0,1000), (1000,1000), (2000,1000)`
- Bouncy bunny (frames 0,14): `(0,0), (256,384)`

### 3. Integration Workflow (`critter-integration.test.js`)
- ✅ Completes full chirpy_bird workflow (Vue → WASM → Spawn)
- ✅ Completes full bouncy_bunny workflow
- ✅ Handles multiple critters without conflicts
- ✅ Verifies animation coordinates match expected Grid layout

## Architecture Verified

The tests confirm that our **dynamic, non-hardcoded system** works correctly:

1. **Vue Component** generates consistent numeric IDs using hash function
2. **WASM Game Engine** receives critter data and uses Grid coordinates
3. **Sprite Animation** uses proper Grid layout instead of horizontal stripes
4. **System is fully extensible** - new critters work without code changes

## Test Files

- `critter-id-mapping.test.js` - Dynamic ID generation tests
- `grid-coordinates.test.js` - Sprite coordinate calculation tests  
- `critter-integration.test.js` - End-to-end workflow tests
- `coordinate_logic.rs` - Rust coordinate calculation tests (requires compilation)

## Running Tests

```bash
cd tests
npm install
npm test
```

## Expected Browser Behavior

With the fixed system, you should see logs like:
```
🐶 Loading critter in game engine: Bouncy (ID: bouncy_bunny -> 307)
🐶 Loading critter: ID=307, Name=Bouncy, Species=Bunny
🎭 Spawned Bouncy at (0, 0)
🎬 Animating frame 1/2 (anim sequence: 0) - Grid coords: (0, 0) rect: ...
🎬 Animating frame 2/2 (anim sequence: 14) - Grid coords: (256, 384) rect: ...
```

The critter system is now **fully dynamic and extensible**! 🎉