use bevy::prelude::*;
use bevy_web_asset::WebAssetPlugin;
use wasm_bindgen::prelude::*;
use web_sys::console;
use std::sync::Mutex;
use std::collections::VecDeque;

mod game;
mod systems;
mod components;
mod resources;

use game::{GamePlugin, LoadCritterEvent, SpawnCritterEvent};
use systems::process_click_on_critters;

// Event queues for communication between WASM interface and Bevy
static LOAD_CRITTER_QUEUE: Mutex<VecDeque<LoadCritterEvent>> = Mutex::new(VecDeque::new());
static INTERACTION_QUEUE: Mutex<VecDeque<(String, f32, f32, f32, f32)>> = Mutex::new(VecDeque::new());

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
        .add_plugins(WebAssetPlugin::default())
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#game-canvas".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }))
        .add_plugins(GamePlugin)
        .add_systems(Update, (
            process_load_critter_queue,
            process_interaction_queue,
        ))
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

    #[wasm_bindgen]
    pub fn handle_interaction(&self, interaction_type: &str, x: f32, y: f32, dir_x: f32, dir_y: f32) {
        console::log_1(&format!("🐾 Pet interaction received: {} at ({}, {}) with direction ({}, {})", 
            interaction_type, x, y, dir_x, dir_y).into());
        
        // Queue the interaction for processing by Bevy
        if let Ok(mut queue) = INTERACTION_QUEUE.lock() {
            queue.push_back((interaction_type.to_string(), x, y, dir_x, dir_y));
        }
    }

    #[wasm_bindgen]
    pub fn load_critter(&self, critter_id: u32, name: &str, species: &str) {
        console::log_1(&format!("🐶 Loading critter: ID={}, Name={}, Species={}", 
            critter_id, name, species).into());
        
        // Queue the critter load event for processing by Bevy
        if let Ok(mut queue) = LOAD_CRITTER_QUEUE.lock() {
            queue.push_back(LoadCritterEvent {
                critter_id,
                name: name.to_string(),
                species: species.to_string(),
            });
        }
    }

    #[wasm_bindgen]
    pub fn get_critter_info(&self) -> js_sys::Object {
        // Return current critter information as JS object
        let info = js_sys::Object::new();
        js_sys::Reflect::set(&info, &"id".into(), &1.into()).unwrap();
        js_sys::Reflect::set(&info, &"name".into(), &"Default Critter".into()).unwrap();
        js_sys::Reflect::set(&info, &"species".into(), &"dog".into()).unwrap();
        js_sys::Reflect::set(&info, &"happiness".into(), &0.8.into()).unwrap();
        js_sys::Reflect::set(&info, &"energy".into(), &1.0.into()).unwrap();
        
        info
    }

    #[wasm_bindgen]
    pub fn unload_critter(&self) {
        console::log_1(&"🚪 Unloading current critter".into());
        // Future: cleanup current critter entity
    }
}

// Systems to process the event queues from WASM interface
fn process_load_critter_queue(
    mut load_events: EventWriter<LoadCritterEvent>,
) {
    if let Ok(mut queue) = LOAD_CRITTER_QUEUE.lock() {
        while let Some(event) = queue.pop_front() {
            load_events.write(event);
        }
    }
}

fn process_interaction_queue(
    critter_query: Query<(Entity, &Transform), With<components::Critter>>,
    mut interaction_events: EventWriter<game::CritterInteractionEvent>,
) {
    if let Ok(mut queue) = INTERACTION_QUEUE.lock() {
        while let Some((interaction_type, x, y, _dir_x, _dir_y)) = queue.pop_front() {
            // Convert screen coordinates to world coordinates and find critter
            let click_position = Vec2::new(x, y);
            
            // Find the closest critter to the click position
            for (entity, transform) in &critter_query {
                let critter_pos = transform.translation.xy();
                let critter_size = 50.0; // Clickable area radius
                
                if click_position.distance(critter_pos) <= critter_size {
                    let interaction_type_enum = match interaction_type.as_str() {
                        "swipe" => game::InteractionType::Swipe(Vec2::ZERO), // Could use dir_x, dir_y
                        "hold" => game::InteractionType::Hold,
                        _ => game::InteractionType::Tap, // Default to tap
                    };
                    
                    interaction_events.write(game::CritterInteractionEvent {
                        critter_entity: entity,
                        interaction_type: interaction_type_enum,
                        position: click_position,
                    });
                    
                    console::log_1(&format!("🎯 {} interaction sent to critter at ({}, {})", 
                        interaction_type, critter_pos.x, critter_pos.y).into());
                    break; // Only interact with the first critter found
                }
            }
        }
    }
}