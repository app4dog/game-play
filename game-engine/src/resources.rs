use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::{Critter, CritterSpecies};

/// Global game assets resource
#[derive(Resource, Default)]
pub struct AssetCollection {
    // Sprites
    pub bird_sprite: Handle<Image>,
    pub bunny_sprite: Handle<Image>,
    
    // Audio
    pub positive_sound: Handle<AudioSource>,
    pub negative_sound: Handle<AudioSource>,
    
    // UI
    pub font: Handle<Font>,
    
    // Materials
    pub default_material: Handle<StandardMaterial>,
}

/// Critter registry for managing different anthropomorphic game characters
#[derive(Resource)]
pub struct CritterRegistry {
    pub available_critters: Vec<CritterTemplate>,
    pub unlocked_critters: Vec<usize>,
}

impl Default for CritterRegistry {
    fn default() -> Self {
        Self {
            available_critters: vec![
                CritterTemplate {
                    name: "Chirpy".to_string(),
                    species: CritterSpecies::Bird,
                    sprite_path: "assets/sprites/bird-animation.png".to_string(),
                    unlock_level: 1,
                    base_stats: CritterStats {
                        speed: 150.0,
                        playfulness: 0.8,
                        obedience: 0.6,
                        energy: 100.0,
                    },
                },
                CritterTemplate {
                    name: "Bouncy".to_string(),
                    species: CritterSpecies::Bunny,
                    sprite_path: "assets/sprites/bunny-sprite-sheet.png".to_string(),
                    unlock_level: 2,
                    base_stats: CritterStats {
                        speed: 120.0,
                        playfulness: 0.9,
                        obedience: 0.7,
                        energy: 120.0,
                    },
                },
            ],
            unlocked_critters: vec![0], // Bird unlocked by default
        }
    }
}

#[derive(Debug, Clone)]
pub struct CritterTemplate {
    pub name: String,
    pub species: CritterSpecies,
    pub sprite_path: String,
    pub unlock_level: u32,
    pub base_stats: CritterStats,
}

#[derive(Debug, Clone)]
pub struct CritterStats {
    pub speed: f32,
    pub playfulness: f32,
    pub obedience: f32,
    pub energy: f32,
}

/// Training progress tracking
#[derive(Resource, Default)]
pub struct TrainingProgress {
    pub completed_words: HashMap<String, u32>, // word -> times practiced
    pub current_difficulty: f32,
    pub training_streak: u32,
}

/// Game configuration
#[derive(Resource)]
pub struct GameConfig {
    pub screen_bounds: Vec2,
    pub pet_spawn_bounds: Vec2,
    pub interaction_sensitivity: f32,
    pub audio_enabled: bool,
    pub vibration_enabled: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            screen_bounds: Vec2::new(1200.0, 800.0), // Larger screen bounds
            pet_spawn_bounds: Vec2::new(500.0, 350.0), // Larger spawn area
            interaction_sensitivity: 1.0,
            audio_enabled: true,
            vibration_enabled: true,
        }
    }
}