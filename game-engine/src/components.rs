use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Critter component - represents an anthropomorphic game character that the pet player interacts with (bird, bunny, etc.)
#[derive(Component, Debug, Clone)]
pub struct Critter {
    pub name: String,
    pub species: CritterSpecies,
    pub personality: CritterPersonality,
    pub energy: f32,
    pub happiness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CritterSpecies {
    Bird,
    Bunny,
    Dog,
    Cat,
}

#[derive(Component, Debug, Clone)]
pub struct CritterPersonality {
    pub playfulness: f32,    // 0.0 - 1.0
    pub curiosity: f32,      // 0.0 - 1.0
    pub obedience: f32,      // 0.0 - 1.0
}

/// Movement component for critters
#[derive(Component)]
pub struct CritterMovement {
    pub velocity: Vec2,
    pub max_speed: f32,
    pub acceleration: f32,
    pub target_position: Option<Vec2>,
}

/// Animation component for sprite sheets
#[derive(Component)]
pub struct SpriteAnimation {
    pub timer: Timer,
    pub frame_count: usize,
    pub current_frame: usize,
    pub repeat: bool,
    pub critter_id: String, // ID to look up frame layout in CritterRegistry
}

/// Interactive area component
#[derive(Component)]
pub struct InteractiveArea {
    pub radius: f32,
    pub interaction_type: InteractionAreaType,
}

#[derive(Debug)]
pub enum InteractionAreaType {
    TreatDispenser,
    ToyArea,
    RestArea,
    TrainingZone,
}

/// UI components
#[derive(Component)]
pub struct GameUI;

#[derive(Component)]
pub struct ScoreDisplay;

#[derive(Component)]
pub struct LevelDisplay;

/// Audio components
#[derive(Component)]
pub struct GameAudioSource {
    pub clip: Handle<bevy::audio::AudioSource>,
    pub volume: f32,
}

/// Training components for vocabulary and behavior
#[derive(Component)]
pub struct TrainingTarget {
    pub word: String,
    pub category: TrainingCategory,
    pub difficulty: f32,
}

#[derive(Debug, Clone)]
pub enum TrainingCategory {
    BasicCommands,  // sit, stay, come
    Objects,        // toy, food, ball
    People,         // mom, dad, family names
    Emotions,       // happy, sad, excited
}