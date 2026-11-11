// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! This crate's shader layout, which `build.rs` precompiles the shader
//! against and the pipeline is built from. It depends on nothing but the
//! layout crate, as the build script includes it too.

use screenwide_shader_layout::{
  fragment, Attribute, Binding, ShaderLayout, VertexBuffer, VertexFormat, FILTERING_SAMPLER,
  TEXTURE,
};

/// The constants block, read at a dynamic offset one `RenderConstants` wide,
/// then the label, secondary label, frozen desktop and magnifier source
/// textures, then the linear and point samplers. Binding 3 is unassigned, so
/// the bindings after it keep their numbers. Vertices are `Vertex`: a
/// position already in NDC, a texture coordinate and a kind.
pub(crate) const OSC: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, Binding::DynamicUniform),
    fragment(1, TEXTURE),
    fragment(2, TEXTURE),
    fragment(4, TEXTURE),
    fragment(5, TEXTURE),
    fragment(6, FILTERING_SAMPLER),
    fragment(7, FILTERING_SAMPLER),
  ]],
  vertex_buffers: &[VertexBuffer {
    stride: 24,
    attributes: &[
      Attribute {
        location: 0,
        offset: 0,
        format: VertexFormat::Float32x2,
      },
      Attribute {
        location: 1,
        offset: 8,
        format: VertexFormat::Float32x2,
      },
      Attribute {
        location: 2,
        offset: 16,
        format: VertexFormat::Uint32,
      },
    ],
  }],
};
