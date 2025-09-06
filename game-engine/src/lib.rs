use bevy::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::console;

mod game;
mod systems;
mod components;
mod resources;

use game::GamePlugin;

// Enable better panic messages in development
#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// Main entry point for WASM
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    console::log_1(&"🐕 App4.Dog Game Engine Starting...".into());
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#game-canvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}

// JavaScript interface for game control
#[wasm_bindgen]
pub struct GameEngine {
    // Future: store game state reference
}

#[wasm_bindgen]
impl GameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameEngine {
        console::log_1(&"🎮 GameEngine initialized".into());
        GameEngine {}
    }

    #[wasm_bindgen]
    pub fn start_game(&self) {
        console::log_1(&"🚀 Game starting...".into());
        // Future: trigger game start event
    }

    #[wasm_bindgen]
    pub fn pause_game(&self) {
        console::log_1(&"⏸️ Game paused".into());
        // Future: pause game systems
    }

    #[wasm_bindgen]
    pub fn reset_game(&self) {
        console::log_1(&"🔄 Game reset".into());
        // Future: reset game state
    }
}