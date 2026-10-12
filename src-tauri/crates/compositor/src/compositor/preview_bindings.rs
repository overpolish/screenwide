// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bindings `preview.wgsl` and `annotations.wgsl` declare, all in group 0
//! and seen by the fragment stage, each at its index here. The compositor
//! builds its bind group layout from this list, and the build script reads the
//! same file to precompile the canvas shader against the registers wgpu gives
//! that layout, so the two cannot drift apart. It depends on nothing else in
//! the crate for that reason.

#[derive(Clone, Copy)]
pub(crate) enum PreviewBinding {
  Uniform,
  /// A read-only storage buffer.
  Storage,
  /// A filterable float 2D texture.
  Texture,
  /// A filterable float 2D array texture.
  TextureArray,
  /// A filtering sampler.
  Sampler,
}

pub(crate) const PREVIEW_BINDINGS: [PreviewBinding; 18] = {
  use PreviewBinding::*;
  [
    Uniform,
    Uniform,
    Texture,
    TextureArray,
    Texture,
    Texture,
    Texture,
    Storage,
    Storage,
    Texture,
    Storage,
    Storage,
    Sampler,
    Sampler,
    Storage,
    Texture,
    Texture,
    Texture,
  ]
};
