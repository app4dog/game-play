/* tslint:disable */
/* eslint-disable */
export function main(): void;
export class GameEngine {
  free(): void;
  constructor();
  start_game(): void;
  pause_game(): void;
  reset_game(): void;
  handle_interaction(interaction_type: string, x: number, y: number, dir_x: number, dir_y: number): void;
  load_critter(critter_id: number, name: string, species: string): void;
  get_critter_info(): object;
  unload_critter(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly __wbg_gameengine_free: (a: number, b: number) => void;
  readonly gameengine_new: () => number;
  readonly gameengine_start_game: (a: number) => void;
  readonly gameengine_pause_game: (a: number) => void;
  readonly gameengine_reset_game: (a: number) => void;
  readonly gameengine_handle_interaction: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly gameengine_load_critter: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly gameengine_get_critter_info: (a: number) => number;
  readonly gameengine_unload_critter: (a: number) => void;
  readonly __wbindgen_export_0: (a: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_4: WebAssembly.Table;
  readonly __wbindgen_export_5: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_6: (a: number, b: number) => void;
  readonly __wbindgen_export_7: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
