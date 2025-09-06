use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::game::*;
// use bevy::log::info;
// use bevy::log;

/// Setup camera system
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_collection: ResMut<AssetCollection>,
) {
    // Load sprite sheets
    asset_collection.bird_sprite = asset_server.load("assets/sprites/bird-animation.png");
    asset_collection.bunny_sprite = asset_server.load("assets/sprites/bunny-sprite-sheet.png");
    
    // Load audio
    asset_collection.positive_sound = asset_server.load("assets/audio/positive/yipee.ogg");
    
    // info!("🎨 Game assets loading started");
}

/// Critter movement system
pub fn critter_movement_system(
    time: Res<Time>,
    mut critter_query: Query<(&mut Transform, &mut CritterMovement), With<Critter>>,
) {
    for (mut transform, mut movement) in &mut critter_query {
        // Update position based on velocity
        transform.translation += movement.velocity.extend(0.0) * time.delta_secs();
        
        // Move towards target if set
        if let Some(target) = movement.target_position {
            let direction = (target - transform.translation.xy()).normalize();
            let distance = transform.translation.xy().distance(target);
            
            if distance > 5.0 {
                movement.velocity = direction * movement.max_speed;
            } else {
                movement.velocity = Vec2::ZERO;
                movement.target_position = None;
            }
        }
        
        // Apply some damping
        movement.velocity *= 0.95;
    }
}

/// Critter interaction system - handles real pet interactions with game critters
pub fn critter_interaction_system(
    mut interaction_events: EventReader<CritterInteractionEvent>,
    mut critter_query: Query<(&mut Critter, &mut CritterMovement, &Transform)>,
    mut game_progress_events: EventWriter<GameProgressEvent>,
) {
    for event in interaction_events.read() {
        if let Ok((mut critter, mut movement, transform)) = critter_query.get_mut(event.critter_entity) {
            match event.interaction_type {
                InteractionType::Tap => {
                    // Critter responds to pet's tap/paw interaction
                    critter.happiness += 0.1;
                    movement.target_position = Some(event.position);
                    
                    game_progress_events.write(GameProgressEvent {
                        score_change: 10,
                        achievement: None,
                    });
                }
                InteractionType::Swipe(direction) => {
                    // Critter follows pet's swipe direction
                    movement.velocity = direction.normalize() * movement.max_speed;
                    critter.energy -= 0.05;
                }
                InteractionType::Hold => {
                    // Critter stays in place when pet holds screen
                    movement.velocity = Vec2::ZERO;
                    critter.happiness += 0.05;
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