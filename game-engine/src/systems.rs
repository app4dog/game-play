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
    // Load sprite sheets using relative URLs that work in browser context
    console_log!("🎨 Starting asset loading with relative URLs...");
    
    // Use relative paths - the browser will resolve these correctly
    asset_collection.bird_sprite = asset_server.load("assets/sprites/bird-animation.png");
    console_log!("🐦 Bird sprite handle created: {:?}", asset_collection.bird_sprite);
    
    asset_collection.bunny_sprite = asset_server.load("assets/sprites/bunny-sprite-sheet.png");  
    console_log!("🐰 Bunny sprite handle created: {:?}", asset_collection.bunny_sprite);
    
    // Load audio
    asset_collection.positive_sound = asset_server.load("assets/audio/positive/yipee.ogg");
    
    console_log!("✅ Asset loading initiated with relative URLs");
}

/// Enhanced asset loading status monitoring system with detailed error handling
pub fn monitor_asset_loading(
    asset_server: Res<AssetServer>,
    asset_collection: Res<AssetCollection>,
    mut monitoring_timer: Local<Timer>,
    time: Res<Time>,
) {
    if monitoring_timer.duration().is_zero() {
        *monitoring_timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    }
    
    monitoring_timer.tick(time.delta());
    
    if monitoring_timer.just_finished() {
        let bird_status = asset_server.get_load_state(&asset_collection.bird_sprite);
        let bunny_status = asset_server.get_load_state(&asset_collection.bunny_sprite);
        
        console_log!("📊 Asset Loading Status Check:");
        console_log!("🔍 Bird Status: {:?}, Bunny Status: {:?}", bird_status, bunny_status);
        
        // Enhanced bird sprite status checking
        if let Some(bird_state) = bird_status {
            match bird_state {
                bevy::asset::LoadState::NotLoaded => console_log!("🐦 Bird sprite: ⏳ Not loaded yet"),
                bevy::asset::LoadState::Loading => console_log!("🐦 Bird sprite: 🔄 Loading in progress..."),
                bevy::asset::LoadState::Loaded => console_log!("🐦 Bird sprite: ✅ Loaded successfully"),
                bevy::asset::LoadState::Failed(err) => {
                    console_log!("🐦 Bird sprite: ❌ Failed to load - Error: {:?}", err);
                    console_log!("🔧 Trying to diagnose asset loading issue...");
                    // Add recovery logic here if needed
                }
            }
        } else {
            console_log!("🐦 Bird sprite: ❓ Status unknown");
        }
        
        // Enhanced bunny sprite status checking  
        if let Some(bunny_state) = bunny_status {
            match bunny_state {
                bevy::asset::LoadState::NotLoaded => console_log!("🐰 Bunny sprite: Not loaded"),
                bevy::asset::LoadState::Loading => console_log!("🐰 Bunny sprite: Loading..."),
                bevy::asset::LoadState::Loaded => console_log!("🐰 Bunny sprite: ✅ Loaded successfully"),
                bevy::asset::LoadState::Failed(_) => console_log!("🐰 Bunny sprite: ❌ Failed to load"),
            }
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
        // Find critter template by name
        let template_idx = critter_registry.available_critters.iter()
            .position(|template| template.name.to_lowercase() == event.name.to_lowercase());
        
        if let Some(idx) = template_idx {
            game_state.selected_critter_template = Some(idx);
            console_log!("🐶 Critter {} selected for spawning", event.name);
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
        // Only spawn if we have a selected critter template and no current critter
        if let (Some(template_idx), None) = (game_state.selected_critter_template, game_state.current_critter_id) {
            if let Some(template) = critter_registry.available_critters.get(template_idx) {
                
                // Check if sprite is loaded, use fallback if not
                let (sprite_handle, use_fallback) = match template.species {
                    CritterSpecies::Bird => {
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
                    CritterSpecies::Bunny => {
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
                    },
                    _ => {
                        console_log!("❓ Using fallback sprite");
                        (asset_server.load("assets/sprites/default.png"), true)
                    },
                };
                
                console_log!("🖼️ Spawning sprite at position ({}, {}) with scale 0.5", event.position.x, event.position.y);
                
                // DEBUG: Create a red rectangle as fallback to test rendering
                commands.spawn((
                    Sprite {
                        image: Default::default(),
                        color: Color::srgb(1.0, 0.0, 0.0), // Bright red for debugging
                        custom_size: Some(Vec2::new(100.0, 100.0)), // 100x100 red square
                        ..default()
                    },
                    Transform::from_translation((event.position + Vec2::new(150.0, 0.0)).extend(200.0))
                        .with_scale(Vec3::splat(1.0)),
                ));
                console_log!("🟥 DEBUG: Spawned red square at ({}, {}) for visibility test", event.position.x + 150.0, event.position.y);
                
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
                        name: template.name.clone(),
                        species: template.species.clone(),
                        personality: CritterPersonality {
                            playfulness: template.base_stats.playfulness,
                            curiosity: 0.7,
                            obedience: template.base_stats.obedience,
                        },
                        energy: template.base_stats.energy,
                        happiness: 0.5,
                    },
                    CritterMovement {
                        velocity: {
                            let mut rng = thread_rng();
                            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                            let speed = rng.gen_range(30.0..80.0); // Random movement speed
                            Vec2::new(angle.cos() * speed, angle.sin() * speed)
                        },
                        max_speed: template.base_stats.speed,
                        acceleration: 100.0,
                        target_position: None,
                    },
                    SpriteAnimation {
                        timer: Timer::from_seconds(1.0 / 10.0, TimerMode::Repeating), // 10 FPS
                        frame_count: if template.species == CritterSpecies::Bird { 6 } else { 2 },
                        current_frame: 0,
                        repeat: true,
                    },
                )).id();
                
                game_state.current_critter_id = Some(critter_entity);
                console_log!("🎭 Spawned {} at ({}, {})", template.name, event.position.x, event.position.y);
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
    
    if timer.just_finished() && game_state.current_critter_id.is_none() && game_state.selected_critter_template.is_some() {
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