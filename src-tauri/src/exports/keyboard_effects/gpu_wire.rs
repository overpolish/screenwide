// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

pub(super) const MAX_KEYS: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
