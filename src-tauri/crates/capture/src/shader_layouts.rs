// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! This crate's shader layouts, which `build.rs` precompiles the shaders
//! against and the pipelines are built from. It depends on nothing but the
//! layout crate, as the build script includes it too.

use screenwide_shader_layout::{
  fragment, Binding, Entry, ShaderLayout, Stages, FILTERING_SAMPLER, TEXTURE,
};

/// A piece's place, read by the vertex stage at the piece's own offset,
/// then the piece's picture and the sampler that reads it.
pub(crate) const DESKTOP_COMPOSITOR: ShaderLayout = ShaderLayout {
  groups: &[&[
    Entry {
      binding: 0,
      stages: Stages::VERTEX,
      kind: Binding::DynamicUniform,
    },
    fragment(1, TEXTURE),
    fragment(2, FILTERING_SAMPLER),
  ]],
  vertex_buffers: &[],
};

/// The thumbnail's constants, a frame's one or two planes and the sampler
/// that reads them, for the macOS camera.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) const CAMERA_THUMBNAIL: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, Binding::Uniform),
    fragment(1, TEXTURE),
    fragment(2, TEXTURE),
    fragment(3, FILTERING_SAMPLER),
  ]],
  vertex_buffers: &[],
};
