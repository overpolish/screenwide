// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The layout of the bindings `annotate_overlay.wgsl` declares: the
//! overlay's own uniform and pictures, then the annotation lists and atlas at
//! the bindings `annotations.wgsl` declares. The pipeline is built from it,
//! and the build script includes it too to precompile the shader against it,
//! so it depends on nothing but the layout crate.

use screenwide_shader_layout::{
  fragment, Binding, ShaderLayout, FILTERING_SAMPLER, READ_ONLY_STORAGE, TEXTURE,
};

pub(crate) const ANNOTATE_OVERLAY: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, Binding::Uniform),
    fragment(1, TEXTURE),
    fragment(2, TEXTURE),
    fragment(3, FILTERING_SAMPLER),
    fragment(4, READ_ONLY_STORAGE),
    fragment(7, READ_ONLY_STORAGE),
    fragment(8, READ_ONLY_STORAGE),
    fragment(9, TEXTURE),
    fragment(10, READ_ONLY_STORAGE),
    fragment(11, READ_ONLY_STORAGE),
  ]],
  vertex_buffers: &[],
};
