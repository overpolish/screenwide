// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The mip levels of a person cut out of their background, which the canvas
//! blurs their drop shadow out of (`cut_out_shadow` in `preview.wgsl`).

use super::*;

impl Compositor {
  /// Makes `texture`'s levels after the first again from what was drawn into
  /// it, for a cut-out picture whose shadow the canvas reads from them.
  pub fn fill_mips(&self, texture: &wgpu::Texture) {
    self.mips.fill(self.gpu, texture);
  }
}
