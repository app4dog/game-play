use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::game::*;
use web_sys::console;
use rand::prelude::*;
use wasm_bindgen::JsCast;
// use bevy::log::info;
// use bevy::log;

macro_rules! console_log {
    ($($t:tt)*) => (console::log_1(&format!($($t)*).into()))
}

/// Setup camera system
pub fn setup_camera(mut commands: Commands, game_config: Res<GameConfig>) {
    commands.spawn(Camera2d);
    
    console_log!("📷 Camera setup with bounds: {}x{}", game_config.screen_bounds.x, game_config.screen_bounds.y);
}

/// Setup UI system
pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|parent| {
            // Score display
            parent
                .spawn((
                    Text::new("Score: 0"),
                    TextFont {
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ))
                .insert(ScoreDisplay);
        });
}

/// Asset loading system
pub fn load_game_assets(
    asset_server: Res<AssetServer>,
    mut asset_collection: ResMut<AssetCollection>,
) {
    // Load sprite sheets using HTTPS URLs with WebAssetPlugin
    console_log!("🎨 Starting asset loading with HTTPS URLs...");
    
    // Use HTTPS URLs with WebAssetPlugin configured properly
    asset_collection.bird_sprite = asset_server.load("https://play.app4.dog:9000/assets/sprites/bird-animation.png");
    console_log!("🐦 Bird sprite handle created: {:?}", asset_collection.bird_sprite);
    
    asset_collection.bunny_sprite = asset_server.load("https://play.app4.dog:9000/assets/sprites/bunny-sprite-sheet.png");  
    console_log!("🐰 Bunny sprite handle created: {:?}", asset_collection.bunny_sprite);
    
    // Load audio
    asset_collection.positive_sound = asset_server.load("https://play.app4.dog:9000/assets/audio/positive/yipee.ogg");
    
    console_log!("✅ Asset loading initiated with HTTPS URLs");
}

/// Enhanced asset loading status monitoring system with detailed error handling
pub fn monitor_asset_loading(
    asset_server: Res<AssetServer>,
    asset_collection: Res<AssetCollection>,
    mut monitoring_timer: Local<Timer>,
    mut assets_loaded: Local<bool>,
    time: Res<Time>,
) {
    // Only monitor if assets aren't loaded yet
    if *assets_loaded {
        return;
    }
    
    if monitoring_timer.duration().is_zero() {
        *monitoring_timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    }
    
    monitoring_timer.tick(time.delta());
    
    if monitoring_timer.just_finished() {
        let bird_status = asset_server.get_load_state(&asset_collection.bird_sprite);
        let bunny_status = asset_server.get_load_state(&asset_collection.bunny_sprite);
        
        let bird_loaded = matches!(bird_status, Some(bevy::asset::LoadState::Loaded));
        let bunny_loaded = matches!(bunny_status, Some(bevy::asset::LoadState::Loaded));
        
        // Only log if not both loaded
        if !(bird_loaded && bunny_loaded) {
            console_log!("📊 Asset Loading Status Check:");
            console_log!("🔍 Bird Status: {:?}, Bunny Status: {:?}", bird_status, bunny_status);
        }
        
        // Enhanced bird sprite status checking
        if let Some(bird_state) = bird_status {
            match bird_state {
                bevy::asset::LoadState::NotLoaded => console_log!("🐦 Bird sprite: ⏳ Not loaded yet"),
                bevy::asset::LoadState::Loading => console_log!("🐦 Bird sprite: 🔄 Loading in progress..."),
                bevy::asset::LoadState::Loaded => {
                    if !*assets_loaded {
                        console_log!("🐦 Bird sprite: ✅ Loaded successfully");
                    }
                },
                bevy::asset::LoadState::Failed(err) => {
                    console_log!("🐦 Bird sprite: ❌ Failed to load - Error: {:?}", err);
                    console_log!("🔧 Trying to diagnose asset loading issue...");
                }
            }
        }
        
        // Enhanced bunny sprite status checking  
        if let Some(bunny_state) = bunny_status {
            match bunny_state {
                bevy::asset::LoadState::NotLoaded => console_log!("🐰 Bunny sprite: Not loaded"),
                bevy::asset::LoadState::Loading => console_log!("🐰 Bunny sprite: Loading..."),
                bevy::asset::LoadState::Loaded => {
                    if !*assets_loaded {
                        console_log!("🐰 Bunny sprite: ✅ Loaded successfully");
                    }
                },
                bevy::asset::LoadState::Failed(_) => console_log!("🐰 Bunny sprite: ❌ Failed to load"),
            }
        }
        
        // Mark as loaded when both assets are loaded
        if bird_loaded && bunny_loaded {
            *assets_loaded = true;
            console_log!("🎉 All sprites loaded successfully! Monitoring stopped.");
        }
    }
}

/// Critter movement system with screen wrapping and position tracking
pub fn critter_movement_system(
    time: Res<Time>,
    mut critter_query: Query<(&mut Transform, &mut CritterMovement), With<Critter>>,
    game_config: Res<GameConfig>,
    mut frame_counter: Local<u32>,
) {
    *frame_counter += 1;
    
    for (mut transform, mut movement) in &mut critter_query {
        let old_pos = transform.translation;
        
        // Update position based on velocity
        transform.translation += movement.velocity.extend(0.0) * time.delta_secs();
        
        // Log position every 60 frames (roughly 1 second at 60fps)
        if *frame_counter % 60 == 0 {
            console_log!("📍 Critter position: ({:.1}, {:.1}, {:.1}) velocity: ({:.1}, {:.1})", 
                transform.translation.x, transform.translation.y, transform.translation.z,
                movement.velocity.x, movement.velocity.y);
        }
        
        // Screen wrapping with margins
        let margin = 50.0;
        let half_width = game_config.screen_bounds.x / 2.0;
        let half_height = game_config.screen_bounds.y / 2.0;
        
        let pos = &mut transform.translation;
        
        // Horizontal wrapping (left-right)
        if pos.x > half_width + margin {
            pos.x = -half_width - margin;
        } else if pos.x < -half_width - margin {
            pos.x = half_width + margin;
        }
        
        // Vertical wrapping (top-bottom) 
        if pos.y > half_height + margin {
            pos.y = -half_height - margin;
        } else if pos.y < -half_height - margin {
            pos.y = half_height + margin;
        }
        
        // Move towards target if set (overrides continuous movement)
        if let Some(target) = movement.target_position {
            let direction = (target - transform.translation.xy()).normalize_or_zero();
            let distance = transform.translation.xy().distance(target);
            
            if distance > 5.0 {
                movement.velocity = direction * movement.max_speed;
            } else {
                movement.velocity = Vec2::ZERO;
                movement.target_position = None;
                
                // Resume random movement after reaching target
                let mut rng = thread_rng();
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let speed = rng.gen_range(30.0..80.0);
                movement.velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            }
        }
        
        // Occasionally change direction for more interesting movement
        if thread_rng().gen_ratio(1, 180) { // ~1/3 chance per second at 60fps
            let mut rng = thread_rng();
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(30.0..80.0);
            movement.velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        }
    }
}

/// Critter interaction system - handles real pet interactions with game critters
pub fn critter_interaction_system(
    mut commands: Commands,
    mut interaction_events: EventReader<CritterInteractionEvent>,
    critter_query: Query<(Entity, &Critter, &Transform)>,
    mut game_progress_events: EventWriter<GameProgressEvent>,
    mut game_state: ResMut<GameState>,
) {
    for event in interaction_events.read() {
        if let Ok((entity, critter, transform)) = critter_query.get(event.critter_entity) {
            match event.interaction_type {
                InteractionType::Tap => {
                    // When critter is tapped, it disappears and gives points
                    commands.entity(entity).despawn();
                    
                    // Clear current critter from game state if it was this one
                    if game_state.current_critter_id == Some(entity) {
                        game_state.current_critter_id = None;
                    }
                    
                    game_progress_events.write(GameProgressEvent {
                        score_change: 50, // Higher score for successfully catching a critter
                        achievement: Some(format!("{} caught!", critter.name)),
                    });
                    
                    console_log!("🎯 {} was caught and disappeared!", critter.name);
                }
                InteractionType::Swipe(_) => {
                    // Swipe still makes critters disappear but gives fewer points
                    commands.entity(entity).despawn();
                    
                    if game_state.current_critter_id == Some(entity) {
                        game_state.current_critter_id = None;
                    }
                    
                    game_progress_events.write(GameProgressEvent {
                        score_change: 25,
                        achievement: None,
                    });
                    
                    console_log!("💨 {} was swiped away!", critter.name);
                }
                InteractionType::Hold => {
                    // Hold interaction also removes critter
                    commands.entity(entity).despawn();
                    
                    if game_state.current_critter_id == Some(entity) {
                        game_state.current_critter_id = None;
                    }
                    
                    game_progress_events.write(GameProgressEvent {
                        score_change: 30,
                        achievement: None,
                    });
                    
                    console_log!("✋ {} was held and disappeared!", critter.name);
                }
            }
        }
    }
}

/// Game state management system
pub fn game_state_system(
    mut game_state: ResMut<GameState>,
    mut game_progress_events: EventReader<GameProgressEvent>,
) {
    for event in game_progress_events.read() {
        game_state.score = (game_state.score as i32 + event.score_change).max(0) as u32;
        
        // Level progression
        let new_level = (game_state.score / 100) + 1;
        if new_level > game_state.level {
            game_state.level = new_level;
            // info!("🎉 Level up! New level: {}", game_state.level);
        }
        
        if let Some(achievement) = &event.achievement {
            // info!("🏆 Achievement unlocked: {}", achievement);
        }
    }
}

/// UI update system
pub fn ui_update_system(
    game_state: Res<GameState>,
    mut score_query: Query<&mut Text, With<ScoreDisplay>>,
) {
    if game_state.is_changed() {
        for mut text in &mut score_query {
            text.0 = format!("Score: {} | Level: {}", game_state.score, game_state.level);
        }
    }
}

/// Critter loading system - handles selection of critter type from Vue frontend
pub fn critter_loading_system(
    mut load_events: EventReader<LoadCritterEvent>,
    mut game_state: ResMut<GameState>,
    critter_registry: Res<CritterRegistry>,
) {
    for event in load_events.read() {
        // Find critter by name in catalog
        let critter_id = critter_registry.catalog.critters.iter()
            .find(|(_, critter)| critter.name.to_lowercase() == event.name.to_lowercase())
            .map(|(id, _)| id.clone());
        
        if let Some(id) = critter_id {
            game_state.selected_critter_id = Some(id.clone());
            console_log!("🐶 Critter {} (ID: {}) selected for spawning", event.name, id);
        } else {
            console_log!("⚠️ Unknown critter: {}", event.name);
        }
    }
}

/// Random critter spawning system
pub fn critter_spawning_system(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnCritterEvent>,
    mut game_state: ResMut<GameState>,
    critter_registry: Res<CritterRegistry>,
    asset_collection: Res<AssetCollection>,
    asset_server: Res<AssetServer>,
) {
    for event in spawn_events.read() {
        // Only spawn if we have a selected critter ID and no current critter
        if let (Some(ref critter_id), None) = (&game_state.selected_critter_id, game_state.current_critter_id) {
            if let Some(critter_data) = critter_registry.catalog.critters.get(critter_id) {
                
                // Check if sprite is loaded, use fallback if not
                let (sprite_handle, use_fallback) = match critter_data.species {
                    critter_keeper::CritterSpecies::Bird => {
                        let handle = asset_collection.bird_sprite.clone();
                        let status = asset_server.get_load_state(&handle);
                        console_log!("🐦 Using bird sprite handle, status: {:?}", status);
                        
                        match status {
                            Some(bevy::asset::LoadState::Loaded) => (handle, false),
                            Some(bevy::asset::LoadState::Failed(_)) => {
                                console_log!("🐦 Bird sprite failed to load, using fallback");
                                (handle, true)
                            },
                            _ => (handle, false), // Still loading or not started
                        }
                    },
                    critter_keeper::CritterSpecies::Bunny => {
                        let handle = asset_collection.bunny_sprite.clone();
                        let status = asset_server.get_load_state(&handle);
                        console_log!("🐰 Using bunny sprite handle, status: {:?}", status);
                        
                        match status {
                            Some(bevy::asset::LoadState::Loaded) => (handle, false),
                            Some(bevy::asset::LoadState::Failed(_)) => {
                                console_log!("🐰 Bunny sprite failed to load, using fallback");
                                (handle, true)
                            },
                            _ => (handle, false), // Still loading or not started
                        }
                    }
                };
                
                console_log!("🖼️ Spawning sprite at position ({}, {}) with scale 0.5", event.position.x, event.position.y);
                
                // Spawn critter entity with maximum visibility  
                let critter_entity = commands.spawn((
                    Sprite {
                        image: if use_fallback { Default::default() } else { sprite_handle },
                        color: if use_fallback { 
                            Color::srgb(0.0, 1.0, 1.0) // Bright cyan for fallback sprite
                        } else { 
                            Color::srgb(1.0, 1.0, 1.0) // White for normal sprite
                        },
                        custom_size: Some(Vec2::new(200.0, 200.0)), // Force size
                        ..default()
                    },
                    Transform::from_translation(event.position.extend(100.0)) // Much higher Z for visibility
                        .with_scale(Vec3::splat(1.0)), // Full scale for maximum visibility
                    Critter {
                        name: critter_data.name.clone(),
                        species: match critter_data.species {
                            critter_keeper::CritterSpecies::Bird => CritterSpecies::Bird,
                            critter_keeper::CritterSpecies::Bunny => CritterSpecies::Bunny,
                        },
                        personality: CritterPersonality {
                            playfulness: critter_data.stats.happiness_boost,
                            curiosity: 0.7,
                            obedience: 0.6, // Default value
                        },
                        energy: critter_data.stats.energy,
                        happiness: 0.5,
                    },
                    CritterMovement {
                        velocity: {
                            let mut rng = thread_rng();
                            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                            let speed = rng.gen_range(30.0..80.0); // Random movement speed
                            Vec2::new(angle.cos() * speed, angle.sin() * speed)
                        },
                        max_speed: critter_data.stats.base_speed,
                        acceleration: 100.0,
                        target_position: None,
                    },
                    SpriteAnimation {
                        timer: Timer::from_seconds(1.0 / 10.0, TimerMode::Repeating), // 10 FPS
                        frame_count: critter_data.sprite.frame_layout.frame_count as usize,
                        current_frame: 0,
                        repeat: true,
                    },
                )).id();
                
                game_state.current_critter_id = Some(critter_entity);
                console_log!("🎭 Spawned {} at ({}, {})", critter_data.name, event.position.x, event.position.y);
            }
        }
    }
}

/// Auto-spawning system - randomly spawns critters every few seconds
pub fn auto_spawn_system(
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut spawn_events: EventWriter<SpawnCritterEvent>,
    game_state: Res<GameState>,
    game_config: Res<GameConfig>,
) {
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(3.0, TimerMode::Repeating); // Spawn every 3 seconds
    }
    
    timer.tick(time.delta());
    
    if timer.just_finished() && game_state.current_critter_id.is_none() && game_state.selected_critter_id.is_some() {
        let mut rng = thread_rng();
        
        // ALWAYS spawn at center for debugging
        let x = 0.0;
        let y = 0.0;
        
        console_log!("🎯 FORCED CENTER SPAWN at (0, 0) for debugging");
        
        spawn_events.write(SpawnCritterEvent {
            position: Vec2::new(x, y),
        });
        
        console_log!("🎲 Auto-spawning critter at random position ({}, {})", x, y);
    }
}

/// Click detection system - finds which critter (if any) was clicked based on position
pub fn process_click_on_critters(
    click_position: Vec2,
    critter_query: Query<(Entity, &Transform), With<Critter>>,
    mut interaction_events: EventWriter<CritterInteractionEvent>,
) {
    for (entity, transform) in &critter_query {
        let critter_pos = transform.translation.xy();
        let critter_size = 50.0; // Approximate clickable area radius (adjustable)
        
        if click_position.distance(critter_pos) <= critter_size {
            interaction_events.write(CritterInteractionEvent {
                critter_entity: entity,
                interaction_type: InteractionType::Tap,
                position: click_position,
            });
            
            console_log!("🎯 Click detected on critter at ({}, {})", critter_pos.x, critter_pos.y);
            return; // Only interact with the first critter found
        }
    }
}

/// Sprite animation system - handles frame-by-frame sprite sheet animation
pub fn sprite_animation_system(
    time: Res<Time>,
    mut animation_query: Query<(&mut SpriteAnimation, &mut Sprite), With<Critter>>,
) {
    for (mut animation, mut sprite) in &mut animation_query {
        animation.timer.tick(time.delta());
        
        if animation.timer.just_finished() {
            // Move to next frame
            animation.current_frame = (animation.current_frame + 1) % animation.frame_count;
            
            // Calculate texture coordinates for sprite sheet (horizontal layout)
            let frame_width = 1.0 / animation.frame_count as f32;
            let offset_x = animation.current_frame as f32 * frame_width;
            
            // For bird (3000x2000 image with 6 frames): each frame is 500x2000
            // For bunny (512x512 image with 2 frames): each frame is 256x512  
            let (image_width, image_height, frame_pixel_width) = if animation.frame_count == 6 {
                // Bird sprite sheet
                (3000.0, 2000.0, 500.0)
            } else {
                // Bunny sprite sheet (or other)
                (512.0, 512.0, 256.0)
            };
            
            // Set the rect to show only the current frame
            sprite.rect = Some(Rect {
                min: Vec2::new(offset_x * image_width, 0.0),
                max: Vec2::new((offset_x + frame_width) * image_width, image_height),
            });
            
            console_log!("🎬 Animating frame {}/{} - rect: {:?}", 
                animation.current_frame + 1, 
                animation.frame_count,
                sprite.rect
            );
        }
    }
}

/// Window size detection system - gets current canvas size and updates game config
pub fn window_resize_system(
    mut game_config: ResMut<GameConfig>,
    mut last_size: Local<Option<Vec2>>,
) {
    // Get canvas size from DOM
    let window = web_sys::window().expect("should have a window");
    let document = window.document().expect("should have a document");
    let canvas = document
        .get_element_by_id("game-canvas")
        .expect("should have game-canvas")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("should be canvas element");
    
    let width = canvas.client_width() as f32;
    let height = canvas.client_height() as f32;
    let current_size = Vec2::new(width, height);
    
    // Only update if size actually changed
    if *last_size != Some(current_size) {
        *last_size = Some(current_size);
        
        // Update screen bounds based on actual canvas size
        game_config.screen_bounds = current_size;
        
        // Update spawn bounds to be slightly smaller than screen bounds
        game_config.pet_spawn_bounds = Vec2::new(width * 0.8, height * 0.8);
        
        console_log!("📏 Canvas size detected: {}x{}, spawn area: {}x{}", 
            width, height, 
            game_config.pet_spawn_bounds.x, game_config.pet_spawn_bounds.y
        );
    }
}