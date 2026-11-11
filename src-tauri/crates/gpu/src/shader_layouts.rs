// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! This crate's shader layouts, which `build.rs` precompiles the shaders
//! against and the pipelines are built from. It depends on nothing but the
//! layout crate, as the build script includes it too.

use screenwide_shader_layout::{fragment, ShaderLayout, FILTERING_SAMPLER, TEXTURE};

/// The level above and the sampler that averages it.
pub(crate) const MIPS: ShaderLayout = ShaderLayout {
  groups: &[&[fragment(0, TEXTURE), fragment(1, FILTERING_SAMPLER)]],
  vertex_buffers: &[],
};
