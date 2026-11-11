// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! This crate's shader layout, which `build.rs` precompiles the shader
//! against and the pipeline is built from. It depends on nothing but the
//! layout crate, as the build script includes it too.

use screenwide_shader_layout::{fragment, Binding, Sample, ShaderLayout};

/// The frame it repeats the edges of.
pub(crate) const EDGE_FILL: ShaderLayout = ShaderLayout {
  groups: &[&[fragment(
    0,
    Binding::Texture {
      sample: Sample::UnfilterableFloat,
      array: false,
    },
  )]],
  vertex_buffers: &[],
};
