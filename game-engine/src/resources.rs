use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::{Critter, CritterSpecies};
use critter_keeper::{CritterCatalog, CritterConfig};

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

/// Critter registry for managing different anthropomorphic game characters using critter-keeper
#[derive(Resource)]
pub struct CritterRegistry {
    pub catalog: CritterCatalog,
    pub config: CritterConfig,
    pub unlocked_critters: Vec<String>, // Now using critter IDs instead of indices
}

impl CritterRegistry {
    pub fn from_ron(catalog_ron: &str, base_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let catalog: CritterCatalog = ron::from_str(catalog_ron)?;
        let config = CritterConfig::new(base_url, "critters/catalog.ron".to_string());
        
        Ok(Self {
            catalog,
            config,
            unlocked_critters: vec!["chirpy_bird".to_string()], // Bird unlocked by default
        })
    }
    
    pub fn get_available_critters(&self) -> Vec<String> {
        self.catalog.critters.keys().cloned().collect()
    }
    
    pub fn is_unlocked(&self, critter_id: &str) -> bool {
        self.unlocked_critters.contains(&critter_id.to_string())
    }
}

impl Default for CritterRegistry {
    fn default() -> Self {
        // Create a minimal catalog for fallback
        let catalog_ron = r#"
        (
            critters: {
                "chirpy_bird": (
                    id: "chirpy_bird",
                    name: "Chirpy",
                    species: Bird,
                    sprite: (
                        path: "assets/sprites/bird-animation.png",
                        frame_layout: (
                            image_size: (3000, 2000),
                            frame_count: 6,
                            frame_size: (1000, 1000),
                            layout: Grid(cols: 3, rows: 2),
                        ),
                        animations: {
                            "idle": (
                                frames: [0, 1, 2, 3, 4, 5],
                                fps: 8.0,
                                looping: true,
                            ),
                        },
                    ),
                    motion: (
                        movement_type: Wander(radius: 300.0, center_attraction: 0.4),
                        path_behavior: Bounce,
                        speed_variation: (0.7, 1.3),
                        direction_change_frequency: 1.5,
                    ),
                    stats: (
                        base_speed: 150.0,
                        energy: 100.0,
                        happiness_boost: 0.2,
                        size_multiplier: 1.0,
                    ),
                ),
            }
        )
        "#;
        
        Self::from_ron(catalog_ron, "assets/".to_string())
            .expect("Failed to create default critter registry")
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