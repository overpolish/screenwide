// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
