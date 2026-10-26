// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// One draw call: a span of the vertex buffer plus the constants it needs.
/// The base scene run carries no chrome colours, so it uses the defaults.
pub(crate) struct Segment {
  pub start: u32,
  pub count: u32,
  pub action_fills: [[f32; 4]; 2],
  pub chrome: [f32; 4],
  pub chrome_outline: [f32; 4],
  pub label: Option<wgpu::TextureView>,
  /// Bound at `t1`. Only the ruler's tolerance notice uses it (kinds 15/37).
  pub secondary: Option<wgpu::TextureView>,
}

impl Segment {
  pub(crate) fn base(start: usize, end: usize) -> Option<Self> {
    (end > start).then(|| Self {
      start: start as u32,
      count: (end - start) as u32,
      action_fills: [[0.0; 4]; 2],
      chrome: [0.0; 4],
      chrome_outline: [0.0; 4],
      label: None,
      secondary: None,
    })
  }
}
