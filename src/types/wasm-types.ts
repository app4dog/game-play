/**
 * 🤓 Re-export auto-generated WASM types for easier importing
 * This avoids hardcoded manual types while keeping imports clean
 */

// Re-export the auto-generated GameEngine type
export type { GameEngine } from './wasm/app4dog_game_engine'

// Re-export other auto-generated types if needed
export type { 
  critters_ready,
  get_available_critters,
  send_event_to_bevy,
  send_audio_response,
  submit_camera_frame,
  send_audio_response_to_bevy
} from './wasm/app4dog_game_engine'

// 🤓 Type-safe interfaces for WASM functions that return generic objects
// These supplement the auto-generated types with proper TypeScript interfaces
export interface BluetoothStatus {
  scanning: boolean
  connectedDevices: number
  discoveredDevices: number
  virtualNetworkEnabled: boolean
  errorCounts?: Record<string, number> // Optional property for fallback return value
}