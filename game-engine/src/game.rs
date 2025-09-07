use bevy::prelude::*;
use crate::components::*;
use crate::systems::*;
use crate::resources::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GameState>()
            .init_resource::<CritterRegistry>()
            .init_resource::<AssetCollection>()
            .init_resource::<GameConfig>()
            
            // Startup systems
            .add_systems(Startup, (
                setup_camera,
                setup_ui,
                load_game_assets,
            ))
            
            // Update systems
            .add_systems(Update, (
                critter_loading_system,
                critter_spawning_system,
                auto_spawn_system,
                critter_movement_system,
                critter_interaction_system,
                game_state_system,
                ui_update_system,
                window_resize_system,
                monitor_asset_loading,
            ))
            
            // Events
            .add_event::<CritterInteractionEvent>()
            .add_event::<GameProgressEvent>()
            .add_event::<SpawnCritterEvent>()
            .add_event::<LoadCritterEvent>();
    }
}

#[derive(Resource, Default)]
pub struct GameState {
    pub score: u32,
    pub level: u32,
    pub current_critter_id: Option<Entity>,
    pub is_paused: bool,
    pub game_mode: GameMode,
    pub selected_critter_template: Option<usize>, // Index into CritterRegistry
}

#[derive(Default, Debug, PartialEq)]
pub enum GameMode {
    #[default]
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Event)]
pub struct CritterInteractionEvent {
    pub critter_entity: Entity,
    pub interaction_type: InteractionType,
    pub position: Vec2,
}

#[derive(Event)]
pub struct GameProgressEvent {
    pub score_change: i32,
    pub achievement: Option<String>,
}

#[derive(Debug)]
pub enum InteractionType {
    Tap,
    Swipe(Vec2), // direction
    Hold,
}

#[derive(Event)]
pub struct SpawnCritterEvent {
    pub position: Vec2,
}

#[derive(Event)]
pub struct LoadCritterEvent {
    pub critter_id: u32,
    pub name: String,
    pub species: String,
}