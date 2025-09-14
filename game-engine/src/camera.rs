use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
// No explicit sampler override; default sampler is fine
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Simple logging helpers
macro_rules! console_log { ($($arg:tt)*) => { web_sys::console::log_1(&format!($($arg)*).into()) } }
macro_rules! console_warn { ($($arg:tt)*) => { web_sys::console::warn_1(&format!($($arg)*).into()) } }

#[derive(Resource, Default, Clone)]
pub struct CameraFrame(pub Option<Vec<u8>>); // RGB bytes (web path), or decoded sample

#[derive(Resource, Default, Debug, Clone)]
pub struct CameraStats {
    pub total_frames: u64,
    pub throttled_frames: u64,
    pub last_ts: f64,
}

#[derive(Resource, Debug, Clone)]
pub struct FrameThrottle {
    pub min_interval_ms: f64,
    pub last_emit_ts: f64,
}

impl Default for FrameThrottle {
    fn default() -> Self { Self { min_interval_ms: 100.0, last_emit_ts: 0.0 } }
}

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct NewFrameEvent {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub mirror_x: bool,
    pub scale: f32,
    pub ts: f64,
}

// Thread-local queue to receive frames from JS quickly without blocking Bevy
thread_local! {
    static CAMERA_QUEUE: std::cell::RefCell<Vec<(u32, u32, Vec<u8>, f64)>> = std::cell::RefCell::new(Vec::new());
}

#[wasm_bindgen]
pub fn submit_camera_frame(width: u32, height: u32, data: js_sys::Uint8Array, ts: f64) -> Result<(), JsValue> {
    let mut buf = vec![0u8; data.length() as usize];
    data.copy_to(&mut buf[..]);
    CAMERA_QUEUE.with(|q| q.borrow_mut().push((width, height, buf, ts)));
    Ok(())
}

fn drain_camera_queue(
    mut frame_res: ResMut<CameraFrame>,
    mut stats: ResMut<CameraStats>,
    mut throttle: ResMut<FrameThrottle>,
    mut ev: EventWriter<NewFrameEvent>,
    ctrl: Option<Res<CameraPreviewControl>>,
) {
    CAMERA_QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        for (w, h, data, ts) in q.drain(..) {
            stats.total_frames += 1;
            // Throttle emission to systems if needed
            let emit = if throttle.last_emit_ts <= 0.0 { true } else { (ts - throttle.last_emit_ts) >= throttle.min_interval_ms };
            frame_res.0 = Some(data);
            stats.last_ts = ts;
            if emit {
                throttle.last_emit_ts = ts;
                // Convert RGB -> RGBA for sprite texture
                let rgb = frame_res.0.as_ref().unwrap();
                let mut rgba = vec![0u8; (w as usize * h as usize) * 4];
                let mut j = 0usize;
                for i in (0..rgba.len()).step_by(4) {
                    rgba[i] = rgb[j];
                    rgba[i + 1] = rgb[j + 1];
                    rgba[i + 2] = rgb[j + 2];
                    rgba[i + 3] = 255;
                    j += 3;
                }
                let mirror_x = ctrl.as_ref().map(|c| c.mirror_x).unwrap_or(false);
                let scale = ctrl.as_ref().map(|c| c.scale).unwrap_or(0.5);
                ev.write(NewFrameEvent { rgba, width: w, height: h, mirror_x, scale, ts });
            } else {
                stats.throttled_frames += 1;
            }
        }
    });
}

fn log_camera_stats(stats: Res<CameraStats>) {
    // Lightweight periodic log every 60 frames
    if stats.total_frames > 0 && stats.total_frames % 60 == 0 {
        console_log!(
            "📈 Camera frames: total={}, throttled={}, last_ts={}",
            stats.total_frames, stats.throttled_frames, stats.last_ts
        );
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CameraFrame>()
            .init_resource::<CameraStats>()
            .init_resource::<FrameThrottle>()
            .init_resource::<CameraPreviewControl>()
            .add_event::<NewFrameEvent>();

        #[cfg(feature = "camera_sprite_preview")]
        {
            app.add_systems(Startup, spawn_camera_preview)
               .add_systems(Update, (drain_camera_queue, update_camera_preview_system, kinematics_preprocess, log_camera_stats));
        }

        #[cfg(not(feature = "camera_sprite_preview"))]
        {
            app.add_systems(Update, (drain_camera_queue, kinematics_preprocess, log_camera_stats, fps_overlay_system));
        }
        console_log!("📷 CameraPlugin initialized");
    }
}

// Stub posture recognition preprocessor (to be replaced with real model)
#[derive(Resource, Default, Debug, Clone)]
pub struct PostureRecognitionState {
    pub last_label: String,
    pub frames_processed: u64,
}

// Simple on-screen texture preview for validation
#[derive(Debug, Clone)]
pub enum PreviewAnchor { TopLeft, TopRight, BottomLeft, BottomRight }

#[derive(Resource, Debug, Clone)]
pub struct CameraPreviewControl {
    pub enabled: bool,
    pub scale: f32,
    pub anchor: PreviewAnchor,
    pub margin: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub mirror_x: bool,
}

impl Default for CameraPreviewControl {
    fn default() -> Self {
        Self { enabled: true, scale: 0.5, anchor: PreviewAnchor::TopRight, margin: 12.0, offset_x: 0.0, offset_y: 0.0, mirror_x: false }
    }
}
#[derive(Resource, Default)]
struct CameraPreviewState {
    image: Option<Handle<Image>>,
    entity: Option<Entity>,
    fps_entity: Option<Entity>,
    width: u32,
    height: u32,
    last_ts: f64,
    smoothed_fps: f32,
}


#[cfg(feature = "camera_sprite_preview")]
#[derive(Resource)]
pub struct CameraPreviewHandle(pub Handle<Image>);

#[cfg(feature = "camera_sprite_preview")]
#[derive(Resource)]
pub struct CameraPreviewEntity(pub Entity);

#[cfg(feature = "camera_sprite_preview")]
pub fn spawn_camera_preview(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let mut img = Image::new_fill(
        Extent3d { width: 2, height: 2, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    // leave default sampler
    let tex = images.add(img);
    commands.insert_resource(CameraPreviewHandle(tex.clone()));

    let ent = commands
        .spawn((
            Sprite {
                image: tex.clone(),
                flip_x: false,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(1.0)),
            Visibility::Visible,
            Name::new("CameraPreview"),
        ))
        .id();
    commands.insert_resource(CameraPreviewEntity(ent));
}

#[cfg(feature = "camera_sprite_preview")]
pub fn update_camera_preview_system(
    mut ev: EventReader<NewFrameEvent>,
    handle: Option<Res<CameraPreviewHandle>>,
    mut images: ResMut<Assets<Image>>,
    preview_entity: Option<Res<CameraPreviewEntity>>,
    mut q_transform: Query<&mut Transform>,
    mut q_sprite: Query<&mut Sprite>,
    mut q_visibility: Query<&mut Visibility>,
    ctrl: Option<Res<CameraPreviewControl>>,
    windows: Query<&Window>,
) {
    let Some(handle) = handle else { return; };
    let Some(preview_entity) = preview_entity else { return; };
    
    // Check if preview is enabled
    let enabled = ctrl.as_ref().map(|c| c.enabled).unwrap_or(true);
    
    // Update visibility based on enabled state
    if let Ok(mut vis) = q_visibility.get_mut(preview_entity.0) {
        *vis = if enabled { Visibility::Visible } else { Visibility::Hidden };
    }
    
    // Only process frame updates if enabled
    if !enabled {
        return;
    }
    
    let anchor = ctrl.as_ref().map(|c| c.anchor.clone()).unwrap_or(PreviewAnchor::TopRight);
    let margin = ctrl.as_ref().map(|c| c.margin).unwrap_or(12.0);
    let offx = ctrl.as_ref().map(|c| c.offset_x).unwrap_or(0.0);
    let offy = ctrl.as_ref().map(|c| c.offset_y).unwrap_or(0.0);
    for e in ev.read() {
        // Update image
        if let Some(img) = images.get_mut(&handle.0) {
            *img = Image::new_fill(
                Extent3d { width: e.width, height: e.height, depth_or_array_layers: 1 },
                TextureDimension::D2,
                &e.rgba,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            );
            // leave default sampler
        }

        if let Ok(mut spr) = q_sprite.get_mut(preview_entity.0) {
            spr.flip_x = e.mirror_x;
        }
        if let Ok(mut tf) = q_transform.get_mut(preview_entity.0) {
            tf.scale = Vec3::splat(e.scale.max(0.01));
            if let Ok(win) = windows.single() {
                let w = win.width();
                let h = win.height();
                let (x, y) = match anchor {
                    PreviewAnchor::TopLeft => ( -w * 0.5 + margin + offx,  h * 0.5 - margin + offy),
                    PreviewAnchor::TopRight => ( w * 0.5 - margin + offx,  h * 0.5 - margin + offy),
                    PreviewAnchor::BottomLeft => ( -w * 0.5 + margin + offx, -h * 0.5 + margin + offy),
                    PreviewAnchor::BottomRight => ( w * 0.5 - margin + offx, -h * 0.5 + margin + offy),
                };
                tf.translation.x = x;
                tf.translation.y = y;
                tf.translation.z = 0.0;
            }
        }
    }
}

// FPS overlay retained for both modes
fn fps_overlay_system(
    mut commands: Commands,
    mut state: Local<CameraPreviewState>,
    mut ev: EventReader<NewFrameEvent>,
    mut q_text: Query<&mut Text>,
    mut q_tf: Query<&mut Transform>,
    windows: Query<&Window>,
) {
    for e in ev.read() {
        if state.fps_entity.is_none() {
            let ent = commands
                .spawn((
                    Text::new("FPS: --"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    Visibility::Visible,
                    Name::new("CameraFPS"),
                ))
                .id();
            state.fps_entity = Some(ent);
            state.last_ts = e.ts;
        }
        let dt_ms = (e.ts - state.last_ts).max(1.0);
        let inst_fps = (1000.0 / dt_ms) as f32;
        state.smoothed_fps = if state.smoothed_fps == 0.0 { inst_fps } else { state.smoothed_fps * 0.9 + inst_fps * 0.1 };
        state.last_ts = e.ts;
        if let Some(fps_ent) = state.fps_entity {
            if let Ok(mut text) = q_text.get_mut(fps_ent) {
                text.0 = format!("FPS: {:.1}", state.smoothed_fps);
            }
            if let (Ok(win), Ok(mut tf)) = (windows.single(), q_tf.get_mut(fps_ent)) {
                tf.translation = Vec3::new(win.width() * 0.5 - 80.0, win.height() * 0.5 - 24.0, 1002.0);
            }
        }
    }
}

fn kinematics_preprocess(
    mut state: Local<PostureRecognitionState>,
    mut ev: EventReader<NewFrameEvent>,
    frame: Res<CameraFrame>,
) {
    for _ in ev.read() {
        state.frames_processed += 1;
        // Placeholder: we have frame bytes in frame.0 as RGB; no actual inference here.
        state.last_label = if state.frames_processed % 120 == 0 { "standing".into() } else { "unknown".into() };
        if state.frames_processed % 60 == 0 {
            console_warn!("🤖 posture(recognizer): frames={} label={}", state.frames_processed, state.last_label);
        }
        let _ = &frame; // avoid unused warning
    }
}

