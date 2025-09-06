// Auto-generated TypeScript bindings for App4.Dog Game Engine WASM module

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

export class GameEngine {
  constructor();
  start_game(): void;
  pause_game(): void;
  reset_game(): void;
  handle_interaction(type: string, x: number, y: number, dir_x: number, dir_y: number): void;
  get_game_state(): GameState;
  free(): void;
}

declare module '/game-engine/app4dog_game_engine.js' {
  export default function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<void>;
  export { GameEngine, GameState, CritterInfo };
}