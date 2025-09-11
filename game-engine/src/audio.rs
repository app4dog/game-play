// Audio Platform Integration Plugin - b00t pattern
// Handles native audio playback through TypeScript bridge with enter/exit sound support

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::CustomEvent;

// Import events for user gesture handling
use crate::events::{JsToBevyEvent, SharedSettings};

// Simple console logging macros for WASM
macro_rules! console_log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into())
    };
}

macro_rules! console_warn {
    ($($arg:tt)*) => {
        web_sys::console::warn_1(&format!($($arg)*).into())
    };
}

macro_rules! console_error {
    ($($arg:tt)*) => {
        web_sys::console::error_1(&format!($($arg)*).into())
    };
}

/// Audio files for different game contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioContext {
    /// Sounds when entering game areas
    Enter,
    /// Sounds when exiting/leaving game areas  
    Exit,
    /// UI interaction sounds
    UI,
    /// Critter interaction sounds
    Critter,
    /// Background ambient sounds
    Ambient,
    /// Test/development sounds
    Test,
}

/// Audio playback requests sent from Bevy to TypeScript
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
#[serde(tag = "type")]
pub enum AudioRequest {
    /// Play a specific audio file
    Play {
        request_id: String,
        sound_id: String,
        context: AudioContext,
        volume: f32,
        loop_audio: bool,
    },
    /// Stop currently playing audio
    Stop {
        request_id: String,
        sound_id: Option<String>, // None = stop all
    },
    /// Set global volume
    SetVolume {
        request_id: String,
        volume: f32,
    },
    /// Test audio system
    Test {
        request_id: String,
        test_type: String,
    },
}

/// Audio responses sent from TypeScript back to Bevy
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
#[serde(tag = "type")]
pub enum AudioResponse {
    /// Audio playback completed or failed
    PlayCompleted {
        request_id: String,
        success: bool,
        duration_seconds: Option<f32>,
        error_message: Option<String>,
    },
    /// Audio stopped
    Stopped {
        request_id: String,
        success: bool,
    },
    /// Volume changed
    VolumeChanged {
        request_id: String,
        new_volume: f32,
    },
    /// Test completed
    TestCompleted {
        request_id: String,
        result: String,
    },
}

/// Resource managing audio state and requests
#[derive(Resource)]
pub struct AudioManager {
    /// Currently playing sounds
    pub playing_sounds: HashMap<String, PlayingSound>,
    /// Pending requests waiting for response
    pub pending_requests: HashMap<String, PendingAudioRequest>,
    /// Global volume setting (0.0 to 1.0)
    pub global_volume: f32,
    /// Audio context mappings (sound_id -> file path)
    pub sound_registry: HashMap<String, AudioFileInfo>,
    /// Error state
    pub last_error: Option<AudioError>,
    pub error_count: u32,
    /// Audio gate for user gesture requirement
    pub gesture_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PlayingSound {
    pub sound_id: String,
    pub context: AudioContext,
    pub volume: f32,
    pub started_at: f64,
    pub is_looping: bool,
}

#[derive(Debug, Clone)]
pub struct PendingAudioRequest {
    pub request: AudioRequest,
    pub timestamp: f64,
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
pub struct AudioFileInfo {
    pub file_path: String,
    pub context: AudioContext,
    pub default_volume: f32,
    pub format: AudioFormat,
}

#[derive(Debug, Clone)]
pub enum AudioFormat {
    Mp3,
    Ogg,
    Wav,
    Auto, // Let platform decide
}

#[derive(Debug, Clone)]
pub enum AudioError {
    FileNotFound(String),
    PlaybackFailed(String),
    UnsupportedFormat(String),
    PermissionDenied,
    NetworkError(String),
}

impl Default for AudioManager {
    fn default() -> Self {
        let mut sound_registry = HashMap::new();
        
        // Register standard game sounds
        sound_registry.insert("enter_area".to_string(), AudioFileInfo {
            file_path: "assets/audio/ui/enter_chime.mp3".to_string(),
            context: AudioContext::Enter,
            default_volume: 0.8,
            format: AudioFormat::Mp3,
        });
        
        sound_registry.insert("exit_area".to_string(), AudioFileInfo {
            file_path: "assets/audio/ui/exit_chime.mp3".to_string(),
            context: AudioContext::Exit,
            default_volume: 0.7,
            format: AudioFormat::Mp3,
        });
        
        sound_registry.insert("yipee".to_string(), AudioFileInfo {
            file_path: "assets/audio/positive/yipee.ogg".to_string(),
            context: AudioContext::Test,
            default_volume: 0.8,
            format: AudioFormat::Auto,
        });
        
        sound_registry.insert("button_click".to_string(), AudioFileInfo {
            file_path: "assets/audio/ui/click.mp3".to_string(),
            context: AudioContext::UI,
            default_volume: 0.6,
            format: AudioFormat::Mp3,
        });

        Self {
            playing_sounds: HashMap::new(),
            pending_requests: HashMap::new(),
            global_volume: 1.0,
            sound_registry,
            last_error: None,
            error_count: 0,
            gesture_enabled: false,
        }
    }
}

impl AudioManager {
    /// Generate a unique request ID
    pub fn generate_request_id() -> String {
        format!("audio-{}", js_sys::Date::now() as u64)
    }
    
    /// Play a sound by ID
    pub fn play_sound(&mut self, sound_id: &str, volume: Option<f32>) -> String {
        let request_id = Self::generate_request_id();
        
        if let Some(sound_info) = self.sound_registry.get(sound_id) {
            let effective_volume = volume.unwrap_or(sound_info.default_volume) * self.global_volume;
            
            let request = AudioRequest::Play {
                request_id: request_id.clone(),
                sound_id: sound_id.to_string(),
                context: sound_info.context.clone(),
                volume: effective_volume,
                loop_audio: false,
            };
            
            self.pending_requests.insert(request_id.clone(), PendingAudioRequest {
                request: request.clone(),
                timestamp: js_sys::Date::now(),
                retry_count: 0,
            });
            
            console_log!("🎵 Playing sound: {} (volume: {:.2})", sound_id, effective_volume);
        } else {
            console_warn!("🎵 Sound not found in registry: {}", sound_id);
            self.handle_error(AudioError::FileNotFound(sound_id.to_string()));
        }
        
        request_id
    }
    
    /// Play enter area sound
    pub fn play_enter_sound(&mut self) -> String {
        console_log!("🚪 Playing enter area sound");
        self.play_sound("enter_area", None)
    }
    
    /// Play exit area sound  
    pub fn play_exit_sound(&mut self) -> String {
        console_log!("🚪 Playing exit area sound");
        self.play_sound("exit_area", None)
    }
    
    /// Test audio system
    pub fn test_audio(&mut self) -> String {
        console_log!("🧪 Testing audio system");
        self.play_sound("yipee", Some(0.8))
    }
    
    /// Handle completed audio request
    pub fn handle_response(&mut self, response: AudioResponse) {
        match response {
            AudioResponse::PlayCompleted { request_id, success, duration_seconds, error_message } => {
                if let Some(pending) = self.pending_requests.remove(&request_id) {
                    if success {
                        console_log!("✅ Audio completed: {} ({:.1}s)", 
                            self.get_sound_id_from_request(&pending.request).unwrap_or("unknown".to_string()),
                            duration_seconds.unwrap_or(0.0)
                        );
                        self.error_count = 0; // Reset error count on success
                    } else {
                        let error_msg = error_message.unwrap_or("Unknown error".to_string());
                        console_warn!("❌ Audio failed: {}", error_msg);
                        self.handle_error(AudioError::PlaybackFailed(error_msg));
                    }
                }
            }
            AudioResponse::Stopped { request_id, success } => {
                self.pending_requests.remove(&request_id);
                if success {
                    console_log!("⏹️ Audio stopped: {}", request_id);
                }
            }
            AudioResponse::VolumeChanged { request_id, new_volume } => {
                self.global_volume = new_volume;
                console_log!("🔊 Volume changed: {:.2}", new_volume);
            }
            AudioResponse::TestCompleted { request_id, result } => {
                console_log!("🧪 Audio test completed: {}", result);
            }
        }
    }
    
    fn get_sound_id_from_request(&self, request: &AudioRequest) -> Option<String> {
        match request {
            AudioRequest::Play { sound_id, .. } => Some(sound_id.clone()),
            _ => None,
        }
    }
    
    /// Handle audio errors with backoff
    pub fn handle_error(&mut self, error: AudioError) {
        self.last_error = Some(error.clone());
        self.error_count += 1;
        
        match error {
            AudioError::PermissionDenied => {
                console_warn!("🎵 Audio permission denied - user gesture required");
            }
            AudioError::FileNotFound(file) => {
                console_warn!("🎵 Audio file not found: {}", file);
            }
            AudioError::PlaybackFailed(msg) => {
                console_warn!("🎵 Audio playback failed: {}", msg);
            }
            _ => {
                console_warn!("🎵 Audio error: {:?}", error);
            }
        }
    }
    
    /// Enable audio after user gesture
    pub fn enable_audio_gesture(&mut self) {
        self.gesture_enabled = true;
        console_log!("🎵 Audio enabled after user gesture");
    }
}

/// System to dispatch audio requests to TypeScript
pub fn dispatch_audio_requests(
    mut audio_requests: EventReader<AudioRequest>,
    mut audio_manager: ResMut<AudioManager>,
) {
    for request in audio_requests.read() {
        // Check if user gesture is required for audio requests
        if !audio_manager.gesture_enabled {
            console_warn!("🎵 Audio request blocked - waiting for user gesture");
            audio_manager.handle_error(AudioError::PermissionDenied);
            continue;
        }
        
        if let Err(e) = send_audio_request_to_js(request) {
            console_error!("Failed to send audio request to JS: {:?}", e);
            audio_manager.handle_error(AudioError::PlaybackFailed(format!("JS dispatch failed: {:?}", e)));
        }
    }
}

/// System to handle user gesture events and enable audio
pub fn handle_user_gesture(
    mut js_events: EventReader<JsToBevyEvent>,
    mut audio_manager: ResMut<AudioManager>,
) {
    for event in js_events.read() {
        if let JsToBevyEvent::UserGesture { request_id, timestamp } = event {
            console_log!("👆 Enabling audio after user gesture: {} at {}", request_id, timestamp);
            audio_manager.enable_audio_gesture();
        }
    }
}

/// System to handle audio responses from TypeScript
pub fn handle_audio_responses(
    mut audio_responses: EventReader<AudioResponse>,
    mut audio_manager: ResMut<AudioManager>,
) {
    for response in audio_responses.read() {
        audio_manager.handle_response(response.clone());
    }
}

/// Apply shared settings to audio manager (SFX volume, etc.)
pub fn apply_shared_settings(
    settings: Res<SharedSettings>,
    mut audio_manager: ResMut<AudioManager>,
) {
    if settings.is_changed() {
        audio_manager.global_volume = settings.sfx_volume.clamp(0.0, 1.0);
        console_log!(
            "🎚️ Applied shared settings to audio: sfx_volume={}",
            audio_manager.global_volume
        );
    }
}
/// System to trigger enter/exit sounds based on game state
pub fn audio_context_system(
    // Add your game state queries here
    // For now, we'll add a simple API to trigger sounds
) {
    // This system can listen for game state changes and automatically trigger sounds
    // Example: when entering a new area, trigger enter sound
    // when leaving an area, trigger exit sound
}

/// Send audio request to JavaScript via CustomEvent
fn send_audio_request_to_js(request: &AudioRequest) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window object")?;
    let request_data = serde_json::to_string(request)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    
    let custom_event = CustomEvent::new_with_event_init_dict(
        "bevy-audio-request",
        &{
            let mut init = web_sys::CustomEventInit::new();
            init.set_detail(&JsValue::from_str(&request_data));
            init
        },
    )?;
    
    window.dispatch_event(&custom_event)?;
    console_log!("📤 Dispatched audio request to JS: {}", request_data);
    Ok(())
}

/// WASM interface for JavaScript to send responses back
#[wasm_bindgen]
pub fn send_audio_response_to_bevy(response_json: &str) -> Result<(), JsValue> {
    let response: AudioResponse = serde_json::from_str(response_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse audio response: {}", e)))?;
    
    // Use the same thread-local queue pattern as main event bridge
    INCOMING_AUDIO_RESPONSES.with(|queue| {
        queue.borrow_mut().push(response);
    });
    
    Ok(())
}

// Thread-local queue for incoming audio responses
thread_local! {
    static INCOMING_AUDIO_RESPONSES: std::cell::RefCell<Vec<AudioResponse>> = std::cell::RefCell::new(Vec::new());
}

/// System to poll JavaScript responses and add them to Bevy's event system
pub fn poll_audio_responses(mut response_writer: EventWriter<AudioResponse>) {
    INCOMING_AUDIO_RESPONSES.with(|queue| {
        let mut responses = queue.borrow_mut();
        for response in responses.drain(..) {
            response_writer.write(response);
        }
    });
}

/// Audio Plugin - b00t platform integration pattern
/// Platform-facing audio bridge plugin
/// LFMF: abstracts audio to support web (JS/DOM) today and Android/native later.
pub struct PlatformAudioPlugin;

impl Plugin for PlatformAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add the audio manager resource
            .init_resource::<AudioManager>()
            
            // Add audio events
            .add_event::<AudioRequest>()
            .add_event::<AudioResponse>()
            
            // Add audio systems
            .add_systems(Update, (
                poll_audio_responses,
                handle_user_gesture,
                dispatch_audio_requests,
                handle_audio_responses,
                audio_context_system,
                apply_shared_settings,
            ).chain());

        console_log!("🎵 PlatformAudioPlugin initialized");
    }
}

/// Convenience functions for triggering common audio events
pub fn play_enter_sound(mut audio_requests: EventWriter<AudioRequest>) {
    let request_id = AudioManager::generate_request_id();
    audio_requests.write(AudioRequest::Play {
        request_id,
        sound_id: "enter_area".to_string(),
        context: AudioContext::Enter,
        volume: 0.8,
        loop_audio: false,
    });
}

pub fn play_exit_sound(mut audio_requests: EventWriter<AudioRequest>) {
    let request_id = AudioManager::generate_request_id();
    audio_requests.write(AudioRequest::Play {
        request_id,
        sound_id: "exit_area".to_string(), 
        context: AudioContext::Exit,
        volume: 0.7,
        loop_audio: false,
    });
}

pub fn test_audio_system(mut audio_requests: EventWriter<AudioRequest>) {
    let request_id = AudioManager::generate_request_id();
    audio_requests.write(AudioRequest::Play {
        request_id,
        sound_id: "yipee".to_string(),
        context: AudioContext::Test,
        volume: 0.8,
        loop_audio: false,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_manager_creation() {
        let audio_manager = AudioManager::default();
        assert_eq!(audio_manager.global_volume, 1.0);
        assert!(audio_manager.sound_registry.contains_key("yipee"));
        assert!(audio_manager.sound_registry.contains_key("enter_area"));
        assert!(audio_manager.sound_registry.contains_key("exit_area"));
    }
    
    #[test]
    fn test_audio_request_serialization() {
        let request = AudioRequest::Play {
            request_id: "test-123".to_string(),
            sound_id: "yipee".to_string(),
            context: AudioContext::Test,
            volume: 0.8,
            loop_audio: false,
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: AudioRequest = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            AudioRequest::Play { request_id, sound_id, volume, .. } => {
                assert_eq!(request_id, "test-123");
                assert_eq!(sound_id, "yipee");
                assert_eq!(volume, 0.8);
            }
            _ => panic!("Wrong request type after deserialization"),
        }
    }
    
    #[test]
    fn test_audio_plugin_integration() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AudioPlugin,
        ));

        // Test that audio manager is properly initialized
        let audio_manager = app.world().resource::<AudioManager>();
        assert!(audio_manager.sound_registry.len() > 0);
        
        // Test audio request event
        let request = AudioRequest::Test {
            request_id: "test-integration".to_string(),
            test_type: "plugin_test".to_string(),
        };
        
        app.world_mut().send_event(request);
        app.update();
        
        // Verify the event was processed (pending request tracking)
        let audio_manager = app.world().resource::<AudioManager>();
        assert!(audio_manager.error_count == 0); // No errors during test
    }
}
