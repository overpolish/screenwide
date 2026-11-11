// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The layout of the bindings `audio_ribbon.wgsl` declares: its constants
//! and the levels it draws. The pipeline is built from it, and the build
//! script includes it too to precompile the shader against it, so it
//! depends on nothing but the layout crate.

use screenwide_shader_layout::{fragment, Binding, Sample, ShaderLayout};

pub(crate) const AUDIO_RIBBON: ShaderLayout = ShaderLayout {
  groups: &[&[
    fragment(0, Binding::Uniform),
    fragment(
      1,
      Binding::Texture {
        sample: Sample::UnfilterableFloat,
        array: false,
      },
    ),
  ]],
  vertex_buffers: &[],
};
