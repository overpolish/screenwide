// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rendering a recording into a finished video: the frames decoded, the
//! canvas, cursor, keyboard, annotations and camera with its effects drawn
//! on the shared GPU compositor, encoded, and the audio mixed in. The editor
//! preview reads frames through the same decoders, so what the export draws
//! matches what the preview showed.

pub mod camera_baked;
pub mod camera_effects;
pub mod cursor_export;
pub mod decode;
pub mod held_surfaces;
pub mod media_preview;
pub mod pin_cache;
pub mod pin_paths;
pub mod recording_model;
pub mod scenes;
pub mod track_selection;
