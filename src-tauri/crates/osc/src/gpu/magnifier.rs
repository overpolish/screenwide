// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The crop magnifier as a native surface hands it over, read by the region
//! OSC's lens and the editor workspace's loupe alike.

/// The twin of `ScreenwideRegionMagnifier`: the layer it reads, the anchor
/// and the shown part of that picture as shares of it, and the box in
/// drawable pixels.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeMagnifier {
  pub active: u32,
  pub pane_index: u32,
  pub layer_id: u32,
  pub sample_camera: u32,
  pub edges: u32,
  pub light_mode: u32,
  pub sample: [f32; 2],
  pub source_min: [f32; 2],
  pub source_max: [f32; 2],
  pub box_origin: [i32; 2],
  pub box_size: [u32; 2],
}
