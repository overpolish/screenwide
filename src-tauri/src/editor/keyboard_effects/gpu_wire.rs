// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

pub(super) const MAX_KEYS: usize = 8;

/// Per-key centre sentinel: the key follows the overlay's centre.
pub(crate) const KEY_CENTER_INHERIT: f32 = -1.0;
/// Per-key centre sentinel: the key keeps the bottom-centre default even when
/// the overlay's centre has been moved elsewhere.
pub(crate) const KEY_CENTER_DEFAULT: f32 = -2.0;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct KeyboardKey {
  pub key_code: u16,
  pub modifier_mask: u32,
  pub visible: u32,
  pub progress: f32,
  pub alpha: f32,
  pub scale: f32,
  pub layout_progress: f32,
  pub slot: u32,
  pub layout_from_mask: u32,
  pub layout_to_mask: u32,
  /// Normalized canvas centre for this key's shortcut group, letting a badge
  /// that was manually placed finish its animation at its own spot while a
  /// differently placed group is on screen. Non-negative is an explicit
  /// centre; the negative sentinels select the overlay centre or the default.
  pub center_x: f32,
  pub center_y: f32,
  /// This key's group size relative to the overlay's requested scale, so a
  /// differently sized group keeps its size without polluting `scale`, which
  /// stays the pure pop-animation scale the renderer's motion blur compares
  /// against the spring curve.
  pub scale_ratio: f32,
}

impl Default for KeyboardKey {
  fn default() -> Self {
    Self {
      key_code: 0,
      modifier_mask: 0,
      visible: 0,
      progress: 0.0,
      alpha: 0.0,
      scale: 0.0,
      layout_progress: 0.0,
      slot: 0,
      layout_from_mask: 0,
      layout_to_mask: 0,
      center_x: KEY_CENTER_INHERIT,
      center_y: KEY_CENTER_INHERIT,
      scale_ratio: 1.0,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct KeyboardOverlay {
  pub key_count: u32,
  pub animation: u32,
  pub appearance: u32,
  pub scale: f32,
  pub progress: f32,
  /// Recording-wide maximum shortcut width in the 20px-tall KBD design
  /// coordinate space. The GPU uses this to keep retained live resizes inside
  /// the current canvas without waiting for the timeline to be re-evaluated.
  pub maximum_width: f32,
  /// Unclamped user setting. `scale` can already be fitted for the canvas that
  /// produced this payload; retained resizes need the original ceiling so the
  /// keyboard can grow again when space is restored.
  pub requested_scale: f32,
  /// Normalized canvas centre. Negative values retain the default bottom-centre position.
  pub center_x: f32,
  pub center_y: f32,
  pub keys: [KeyboardKey; MAX_KEYS],
}

impl Default for KeyboardOverlay {
  fn default() -> Self {
    Self {
      key_count: 0,
      animation: Self::ANIMATION_NONE,
      appearance: Self::APPEARANCE_LIGHT,
      scale: 1.0,
      progress: 1.0,
      maximum_width: 0.0,
      requested_scale: 0.0,
      center_x: -1.0,
      center_y: -1.0,
      keys: [KeyboardKey::default(); MAX_KEYS],
    }
  }
}

impl KeyboardOverlay {
  pub(crate) const ANIMATION_POP: u32 = 0;
  pub(crate) const ANIMATION_FADE: u32 = 1;
  pub(crate) const ANIMATION_NONE: u32 = 2;
  pub(crate) const APPEARANCE_DARK: u32 = 0;
  pub(crate) const APPEARANCE_LIGHT: u32 = 1;
}
