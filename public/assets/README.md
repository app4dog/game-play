# Game Assets

This directory contains all game assets migrated from puppyplay-godot-droid:

## Structure
- `sprites/` - Game sprite sheets and images
- `audio/` - Sound effects and audio files  
  - `entry.ogg` -> `positive/yipee.ogg`
  - `success.ogg` -> `positive/yipee.ogg`
  - `positive/` - Positive feedback sounds (e.g. `yipee.ogg`)
- `fonts/` - Custom fonts for the game
- `models/` - 3D models (if needed for future features)

## Migration Notes
Assets were migrated from the original Godot project to work with the new Quasar/Capacitor + Rust/Bevy WASM architecture.
