// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! This crate's shader layout, which `build.rs` precompiles the shader
//! against and the pipeline is built from. It depends on nothing but the
//! layout crate, as the build script includes it too.

use screenwide_shader_layout::{fragment, Binding, ShaderLayout, FILTERING_SAMPLER, TEXTURE};

/// What every camera effects pass binds: its source, the matte and what is
/// behind as textures, the sampler, and the uniform.
pub(crate) const CAMERA_EFFECTS: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, TEXTURE),
    fragment(1, TEXTURE),
    fragment(2, TEXTURE),
    fragment(3, FILTERING_SAMPLER),
    fragment(4, Binding::Uniform),
  ]],
  vertex_buffers: &[],
};
