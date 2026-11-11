// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! This crate's shader layouts, which `build.rs` precompiles the shaders
//! against and the pipelines are built from. It depends on nothing but the
//! layout crate, as the build script includes it too.

use screenwide_shader_layout::{fragment, Binding, ShaderLayout, FILTERING_SAMPLER, TEXTURE};

/// The checkerboard's colours and square size.
pub(crate) const BACKDROP: ShaderLayout = ShaderLayout {
  groups: &[&[fragment(0, Binding::Uniform)]],
  vertex_buffers: &[],
};

/// The lens, the picture it magnifies and the sampler that reads it.
pub(crate) const LOUPE: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, Binding::Uniform),
    fragment(1, TEXTURE),
    fragment(2, FILTERING_SAMPLER),
  ]],
  vertex_buffers: &[],
};
