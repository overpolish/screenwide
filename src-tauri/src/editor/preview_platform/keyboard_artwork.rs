// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The keyboard-shortcut artwork strip: one 20pt-tall run of rounded key caps,
//! rasterised by the platform once per appearance, density and shortcut, then
//! uploaded and animated entirely on the GPU. The platform's rasteriser sets
//! the caps' labels in its own key names and type.

mod cache;
#[cfg(test)]
mod tests;
#[cfg_attr(target_os = "macos", allow(dead_code))]
mod visible_bounds;

use super::surface::keyboard_raster::rasterize_keyboard;
use crate::editor::keyboard_effects::{KeyboardKey, KeyboardOverlay};

const MAX_KEYS: usize = 8;
/// A key cap's height in points, which the strip is rasterised at.
pub(crate) const DESIGN_HEIGHT: f64 = 20.0;
const CACHE_ENTRIES: usize = 64;
const CACHE_BYTES: usize = 64 * 1024 * 1024;

/// The twin of `Keyboard` in `preview.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct KeyboardConstants {
  pub(crate) dimensions: [u32; 4],
  pub(crate) animation: [f32; 4],
  pub(crate) position: [f32; 4],
  pub(crate) key_geometry: [[u32; 4]; MAX_KEYS],
  pub(crate) key_motion: [[f32; 4]; MAX_KEYS],
  pub(crate) key_masks: [[u32; 4]; MAX_KEYS],
  pub(crate) key_position: [[f32; 4]; MAX_KEYS],
}

const _: () = assert!(size_of::<KeyboardConstants>() == 560);

impl Default for KeyboardConstants {
  fn default() -> Self {
    Self {
      dimensions: [0; 4],
      animation: [0.0; 4],
      position: [0.0; 4],
      key_geometry: [[0; 4]; MAX_KEYS],
      key_motion: [[0.0; 4]; MAX_KEYS],
      key_masks: [[0; 4]; MAX_KEYS],
      key_position: [[-1.0, -1.0, 1.0, 0.0]; MAX_KEYS],
    }
  }
}

/// A rasterised strip: premultiplied BGRA rows, its size in pixels, and where
/// each key cap starts and how wide it is.
pub(crate) struct KeyboardRaster {
  pub(crate) pixels: Vec<u8>,
  pub(crate) size: (u32, u32),
  pub(crate) keys: Vec<(u32, u32)>,
}

pub(crate) struct KeyboardArtwork {
  bytes: usize,
  keys: Vec<(u32, u32)>,
  size: (u32, u32),
  pub(crate) view: wgpu::TextureView,
}

#[derive(Default)]
pub(crate) struct KeyboardArtworkCache {
  entries: std::sync::Mutex<std::collections::HashMap<String, std::sync::Arc<KeyboardArtwork>>>,
}

/// How many pixels a design point is rasterised at: enough for the strip's
/// largest animated size on an `output_height` canvas.
pub(crate) fn keyboard_backing_scale(output_height: u32, overlay: &KeyboardOverlay) -> f64 {
  const MAXIMUM_ANIMATED_SCALE: f64 = 1.08;
  let requested = if overlay.requested_scale > 0.0 {
    overlay.requested_scale
  } else {
    overlay.scale
  };
  let pixels = f64::from(output_height)
    * (60.0 / 1080.0)
    * f64::from(requested).max(0.0)
    * MAXIMUM_ANIMATED_SCALE;
  (pixels / DESIGN_HEIGHT).ceil().clamp(12.0, 64.0)
}

pub(crate) fn is_modifier_key(code: u16) -> bool {
  matches!(code, 54..=56 | 58..=63)
}

/// Version-one sidecars stored a modifier mask on the single recorded key.
/// Expanding it here keeps old recordings looking like the grouped shortcut.
fn prepared_shortcut(overlay: &KeyboardOverlay) -> Vec<(u16, KeyboardKey)> {
  let mut prepared = Vec::with_capacity(MAX_KEYS);
  let count = (overlay.key_count as usize).min(MAX_KEYS);
  for state in overlay.keys.iter().take(count) {
    if count == 1 && !is_modifier_key(state.key_code) {
      for (bit, code) in [55_u16, 59, 58, 56, 63].into_iter().enumerate() {
        if state.modifier_mask & (1 << bit) != 0 && prepared.len() < MAX_KEYS {
          prepared.push((code, *state));
        }
      }
    }
    if prepared.len() < MAX_KEYS {
      prepared.push((state.key_code, *state));
    }
  }
  prepared
}
