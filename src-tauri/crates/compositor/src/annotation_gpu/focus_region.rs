// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// How wide the blur reaches at the middle strength, as a share of the
/// picture's shorter side: enough to read as a lens without smearing what is
/// around the box out of recognition. The editor's focus and the live
/// overlay's both measure by it.
pub const FOCUS_WIDEST_SHARE: f32 = 0.013;
/// How far past a box the blur takes to arrive, as a share of the picture's
/// shorter side.
pub const FOCUS_REACH_SHARE: f32 = 0.45;

/// One focus box a frame's blur keeps sharp, in the source's own pixels.
/// The blur is a pass over the drawn canvas, so the box is held beside the
/// arrows rather than drawn with them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusRegion {
  /// The box's top-left and bottom-right corners, in source pixels.
  pub bounds: [f32; 4],
  /// The corner radius in source pixels, a share of the box's shorter side.
  pub radius: f32,
  /// The blur's strength, a step from one to five.
  pub strength: f32,
  /// How present the box is this frame, from its reveal.
  pub presence: f32,
}

/// A focus box as the shaders read it: the twin of `FocusRegion` in
/// `focus_falloff.wgsl`, a 32-byte stride.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuFocusRegion {
  bounds: [f32; 4],
  radius: f32,
  strength: f32,
  presence: f32,
  reference: f32,
}

impl GpuFocusRegion {
  /// `region` with the inverse turn's weight at its middle, which the
  /// editor's depth term measures from; one where the screen is upright.
  pub fn new(region: &FocusRegion, reference: f32) -> Self {
    Self {
      bounds: region.bounds,
      radius: region.radius,
      strength: region.strength,
      presence: region.presence,
      reference,
    }
  }
}
