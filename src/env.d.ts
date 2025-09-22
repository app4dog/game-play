/// <reference types="vite/client" />

// Fix for Quasar TypeScript exports resolution issue in CI builds
declare module 'quasar' {
  // Import all Quasar types and exports
  export * from 'quasar/dist/types/index';
  // Specific exports that are commonly used
  export { useQuasar } from 'quasar/dist/types/composables/use-quasar';
  export { Notify } from 'quasar/dist/types/plugins/notify';
}

// Additional fix for specific Quasar dist files that CI builds reference
declare module 'quasar/dist/quasar.client.js' {
  export * from 'quasar/dist/types/index';
}

// Fallback for any unresolved Quasar modules
declare module 'quasar/*' {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const content: any;
  export = content;
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
declare namespace NodeJS {
  interface ProcessEnv {
    NODE_ENV: string;
    VUE_ROUTER_MODE: 'hash' | 'history' | 'abstract' | undefined;
    VUE_ROUTER_BASE: string | undefined;
  }
}

declare global {
  // Unified WASM interface for all composables
  type WasmInit = (
    input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
  ) => Promise<void>

  // 🤓 Use auto-generated types instead of duplicating manually
  // eslint-disable-next-line @typescript-eslint/consistent-type-imports -- referencing auto-generated WASM types
  type GameEngineApi = import('../types/wasm-types').GameEngine

  interface UnifiedWasmModule {
    // Core WASM module functions (from GameCanvas)
    default: WasmInit
    GameEngine: new () => GameEngineApi
    game_engine?: GameEngineApi
    
    // Event bridge functions
    send_event_to_bevy?: (eventJson: string) => void
    send_js_to_bevy_event?: (eventJson: string) => void
    // Camera: submit raw frame bytes (RGB or YUV bytes)
    submit_camera_frame?: (width: number, height: number, data: Uint8Array, ts: number) => void
    
    // Audio response functions  
    send_audio_response?: (responseJson: string) => void
    send_audio_response_to_bevy?: (responseJson: string) => void
    
    // Critter management functions (for CritterSelection component)
    critters_ready?: () => boolean
    get_available_critters?: () => unknown[]
    
    // Other WASM exports - allow any additional functions
    [key: string]: unknown
  }

  interface Window {
    __A4D_WASM__?: UnifiedWasmModule
    webkitAudioContext?: typeof AudioContext
    // Global kill-switch to disable background music regardless of settings
    __A4D_DISABLE_BGM__?: boolean
    // Global kill-switch to disable all audio (AudioContext) regardless of settings
    __A4D_DISABLE_AUDIO__?: boolean
  }
}

// Ensure this file is treated as a module
export {}
