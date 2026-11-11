// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! This crate's shader layout, which `build.rs` precompiles the shader
//! against and the pipeline is built from. It depends on nothing but the
//! layout crate, as the build script includes it too.

use screenwide_shader_layout::{Binding, Entry, ShaderLayout, Stages, StorageFormat};

/// The mesh's parameters and the picture it writes.
pub(crate) const MESH: ShaderLayout = ShaderLayout {
  groups: &[&[
    Entry {
      binding: 0,
      stages: Stages::COMPUTE,
      kind: Binding::Uniform,
    },
    Entry {
      binding: 1,
      stages: Stages::COMPUTE,
      kind: Binding::StorageTexture {
        format: StorageFormat::Rgba8Unorm,
      },
    },
  ]],
  vertex_buffers: &[],
};
