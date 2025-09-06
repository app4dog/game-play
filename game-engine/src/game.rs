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
            
            // Startup systems
            .add_systems(Startup, (
                setup_camera,
                setup_ui,
                load_game_assets,
            ))
            
            // Update systems
            .add_systems(Update, (
                critter_movement_system,
                critter_interaction_system,
                game_state_system,
                ui_update_system,
            ))
            
            // Events
            .add_event::<CritterInteractionEvent>()
            .add_event::<GameProgressEvent>();
    }
}

#[derive(Resource, Default)]
pub struct GameState {
    pub score: u32,
    pub level: u32,
    pub current_critter_id: Option<Entity>,
    pub is_paused: bool,
    pub game_mode: GameMode,
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