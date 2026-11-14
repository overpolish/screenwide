// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! This crate's shader layouts, which `build.rs` precompiles the shaders
//! against and the pipelines are built from. It depends on nothing but the
//! layout crate, as the build script includes it too.

use screenwide_shader_layout::{
  fragment, Binding, Entry, Sample, ShaderLayout, Stages, FILTERING_SAMPLER, READ_ONLY_STORAGE,
  TEXTURE,
};

const UNIFORM: Binding = Binding::Uniform;
const TEXTURE_ARRAY: Binding = Binding::Texture {
  sample: Sample::Float,
  array: true,
};
const UNFILTERED: Binding = Binding::Texture {
  sample: Sample::UnfilterableFloat,
  array: false,
};

const fn compute(binding: u32, kind: Binding) -> Entry {
  Entry {
    binding,
    stages: Stages::COMPUTE,
    kind,
  }
}

/// The bindings `preview.wgsl` and `annotations.wgsl` declare, each at its
/// index: the canvas, keyboard and their pictures, the annotation lists and
/// atlases, the samplers, and the layers under the lenses and spotlights.
pub(crate) const PREVIEW: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, UNIFORM),
    fragment(1, UNIFORM),
    fragment(2, TEXTURE),
    fragment(3, TEXTURE_ARRAY),
    fragment(4, TEXTURE),
    fragment(5, TEXTURE),
    fragment(6, TEXTURE),
    fragment(7, READ_ONLY_STORAGE),
    fragment(8, READ_ONLY_STORAGE),
    fragment(9, TEXTURE),
    fragment(10, READ_ONLY_STORAGE),
    fragment(11, READ_ONLY_STORAGE),
    fragment(12, FILTERING_SAMPLER),
    fragment(13, FILTERING_SAMPLER),
    fragment(14, READ_ONLY_STORAGE),
    fragment(15, TEXTURE),
    fragment(16, TEXTURE),
    fragment(17, TEXTURE),
    // The layers under a lens: `compositor/lens_under.rs`.
    fragment(18, TEXTURE),
    // What each lens shows: `compositor/lens_ground.rs`.
    fragment(19, TEXTURE),
    // The picture of the background giving way while a scene's blooms in:
    // `compositor/background_turn.rs`.
    fragment(20, TEXTURE),
  ]],
  vertex_buffers: &[],
};

/// The tile binning pass's: the canvas constants and the annotation lists it
/// reads, at the canvas pass's bindings, then the tiles' sets it writes.
pub(crate) const TILES: ShaderLayout = ShaderLayout {
  groups: &[
    &[
      compute(0, UNIFORM),
      compute(7, READ_ONLY_STORAGE),
      compute(8, READ_ONLY_STORAGE),
      compute(10, READ_ONLY_STORAGE),
    ],
    &[compute(0, Binding::Storage { writable: true })],
  ],
  vertex_buffers: &[],
};

/// A redaction pass's record, read at its own offset, the source, the zones,
/// and the cells and rows the passes before it wrote.
pub(crate) const REDACT: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, Binding::DynamicUniform),
    fragment(1, UNFILTERED),
    fragment(2, READ_ONLY_STORAGE),
    fragment(3, UNFILTERED),
    fragment(4, UNFILTERED),
  ]],
  vertex_buffers: &[],
};

/// A frame's two planes and the sampler that reads them, for the macOS
/// export.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) const VIDEO_PLANES: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, TEXTURE),
    fragment(1, TEXTURE),
    fragment(2, FILTERING_SAMPLER),
  ]],
  vertex_buffers: &[],
};

/// A focus chain's level above and the sampler that reads it.
pub(crate) const FOCUS_LEVEL: ShaderLayout = ShaderLayout {
  groups: &[&[fragment(0, TEXTURE), fragment(1, FILTERING_SAMPLER)]],
  vertex_buffers: &[],
};

/// The focus gather's: the chain and its sampler, then its constants and its
/// boxes.
pub(crate) const FOCUS_GATHER: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, TEXTURE),
    fragment(1, FILTERING_SAMPLER),
    fragment(2, UNIFORM),
    fragment(3, READ_ONLY_STORAGE),
  ]],
  vertex_buffers: &[],
};

/// The Gaussian pass's axis and the two layers it reads.
pub(crate) const MARK_BLUR: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, UNIFORM),
    fragment(1, TEXTURE),
    fragment(2, TEXTURE),
  ]],
  vertex_buffers: &[],
};
