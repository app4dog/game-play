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

  interface GameEngineApi {
    start_game(): void
    pause_game(): void
    reset_game(): void
    handle_interaction?(type: string, x: number, y: number, dir_x: number, dir_y: number): void
    load_critter?(critter_id: number, name: string, species: string): void
    load_critter_by_id?(id: string): void
    get_critter_info?(): { id: number; name: string; species: string; happiness: number; energy: number }
    unload_critter?(): void
    free?(): void
  }

  interface UnifiedWasmModule {
    // Core WASM module functions (from GameCanvas)
    default: WasmInit
    GameEngine: new () => GameEngineApi
    
    // Event bridge functions
    send_event_to_bevy?: (eventJson: string) => void
    send_js_to_bevy_event?: (eventJson: string) => void
    
    // Audio response functions  
    send_audio_response?: (responseJson: string) => void
    send_audio_response_to_bevy?: (responseJson: string) => void
    
    // Other WASM exports - allow any additional functions
    [key: string]: unknown
  }

  interface Window {
    __A4D_WASM__?: UnifiedWasmModule
    webkitAudioContext?: typeof AudioContext
    // Global kill-switch to disable background music regardless of settings
    __A4D_DISABLE_BGM__?: boolean
  }
}

// Ensure this file is treated as a module
export {}
