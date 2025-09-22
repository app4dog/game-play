/**
 * 🤓 WASM types for easier importing
 * Clean re-exports from auto-generated WASM bindings (source of truth)
 * 
 * 🚨 CRITICAL: NEVER DUPLICATE AUTO-GENERATED TYPES - ALWAYS DRY, NEVER CRY!
 * 
 * ✅ DO: Import GameEngine from './wasm/app4dog_game_engine' (auto-generated)
 * ❌ DON'T: Manually define GameEngine interface here
 * 
 * The auto-generated types in ./wasm/ are the SINGLE SOURCE OF TRUTH.
 * They are generated from Rust code with #[wasm_bindgen] annotations.
 * Any manual duplication leads to CRY (Continually Repeat Yourself) anti-pattern.
 */

// Import from the auto-generated WASM types (source of truth)
export { GameEngine } from './wasm/app4dog_game_engine'

// Define additional interfaces for WASM return types that use generic 'object'
export interface GameState {
  score: number
  level: number
  is_paused: boolean
  current_critter_id: number | null
}

export interface CritterInfo {
  name: string
  species: string
  happiness: number
  energy: number
}

// 🤓 Type-safe interfaces for WASM functions that return generic objects
// These supplement the auto-generated types with proper TypeScript interfaces
export interface BluetoothStatus {
  scanning: boolean
  connectedDevices: number
  discoveredDevices: number
  virtualNetworkEnabled: boolean
  errorCounts?: Record<string, number> // Optional property for fallback return value
}