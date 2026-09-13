// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! CPU rasterisation and D3D11 upload of the keyboard-shortcut artwork strip.
//! Mirrors the macOS Core Graphics rasteriser so both backends feed the same
//! shader geometry: one 20pt-tall strip of rounded key caps, drawn once per
//! appearance/density/shortcut and animated entirely on the GPU afterwards.

#[path = "keyboard_artwork/cache.rs"]
mod cache;
#[path = "keyboard_artwork/labels.rs"]
mod labels;
#[path = "keyboard_artwork/rasterize.rs"]
mod rasterize;
#[cfg(test)]
#[path = "keyboard_artwork/tests.rs"]
mod tests;
use labels::{key_label, prepared_shortcut};
pub(super) use rasterize::rasterize_keyboard;

use std::{collections::HashMap, ffi::c_void, sync::Mutex};

use windows::{
  core::{Interface, PCWSTR},
  Win32::{
    Foundation::COLORREF,
    Graphics::{
      Direct3D11::{
        ID3D11Device, ID3D11Resource, ID3D11ShaderResourceView, ID3D11Texture2D,
        D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_IMMUTABLE,
      },
      Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
      Gdi::{
        CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject,
        GetTextExtentPoint32W, SelectObject, SetBkMode, SetTextCharacterExtra, SetTextColor,
        TextOutW, ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS,
        DEFAULT_CHARSET, DIB_RGB_COLORS, FF_SWISS, FW_NORMAL, OUT_DEFAULT_PRECIS, TRANSPARENT,
        VARIABLE_PITCH,
      },
    },
  },
};

use crate::editor::keyboard_effects::{KeyboardKey, KeyboardOverlay};

#[path = "keyboard_artwork/icons.rs"]
mod icons;
#[path = "keyboard_artwork/visible_bounds.rs"]
mod visible_bounds;

const MAX_KEYS: usize = 8;
const DESIGN_HEIGHT: f64 = 20.0;
const DESIGN_MINIMUM_WIDTH: f64 = 20.0;
const DESIGN_INSET: f64 = 4.0;
const DESIGN_GAP: f64 = 4.0;
const DESIGN_RADIUS: f64 = 4.0;
const FONT_SIZE: f64 = 13.0;
const FILL_ALPHA: f64 = 0.10;
const TEXT_ALPHA: f64 = 0.85;
const CACHE_ENTRIES: usize = 64;
const CACHE_BYTES: usize = 64 * 1024 * 1024;

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct KeyboardConstants {
  pub(super) dimensions: [u32; 4],
  pub(super) animation: [f32; 4],
  pub(super) position: [f32; 4],
  pub(super) key_geometry: [[u32; 4]; MAX_KEYS],
  pub(super) key_motion: [[f32; 4]; MAX_KEYS],
  pub(super) key_masks: [[u32; 4]; MAX_KEYS],
  pub(super) key_position: [[f32; 4]; MAX_KEYS],
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

pub(super) struct KeyboardRaster {
  pub(super) pixels: Vec<u8>,
  pub(super) size: (u32, u32),
  pub(super) keys: Vec<(u32, u32)>,
}

pub(super) struct KeyboardArtwork {
  _texture: ID3D11Texture2D,
  bytes: usize,
  keys: Vec<(u32, u32)>,
  size: (u32, u32),
  pub(super) view: ID3D11ShaderResourceView,
}

#[derive(Default)]
pub(super) struct KeyboardArtworkCache {
  entries: Mutex<HashMap<String, std::sync::Arc<KeyboardArtwork>>>,
}

pub(super) fn keyboard_backing_scale(output_height: u32, overlay: &KeyboardOverlay) -> f64 {
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

struct TextDevice {
  dc: windows::Win32::Graphics::Gdi::HDC,
  font: windows::Win32::Graphics::Gdi::HFONT,
  old_font: windows::Win32::Graphics::Gdi::HGDIOBJ,
}
