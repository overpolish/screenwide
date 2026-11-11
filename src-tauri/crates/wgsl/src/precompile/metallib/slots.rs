// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where wgpu-hal 30's Metal backend puts a stage's resources and vertex
//! buffers (`create_pipeline_layout` and `create_render_pipeline` in its
//! `metal/device.rs`), which a precompiled library has to read them from.

use naga::back::msl;
use screenwide_shader_layout::{Binding, ShaderLayout, Stages, VertexFormat};

/// The buffer slots wgpu-hal's Metal backend has, the vertex buffers taking
/// them from the last down.
const MAX_BUFFERS: u32 = 31;

/// The slots wgpu-hal 30 gives a stage's resources: buffers, textures and
/// samplers each counted apart, in group and binding order, over only what
/// the stage sees. The buffer after the last holds the sizes of runtime-sized
/// arrays where the stage sees a storage buffer; a vertex stage takes it
/// always, for vertex pulling.
///
/// Buffer accesses are left unchecked, which wgpu does only when asked. A
/// checked access to a runtime-sized storage array reads that array's length
/// from the sizes buffer, and wgpu fills that buffer from the
/// `sized_bindings` it finds while compiling MSL itself; for a precompiled
/// library it has none. naga sets the policy for every buffer at once, so
/// uniform arrays go unchecked too. Indices into the shader's own arrays and
/// texel loads stay checked: neither reads the sizes buffer.
pub(super) fn resources(layout: &ShaderLayout, stage: Stages) -> msl::EntryPointResources {
  let mut resources = msl::EntryPointResources::default();
  let (mut buffers, mut textures, mut samplers) = (0, 0, 0);
  let mut sizes = stage == Stages::VERTEX;
  for (group, entries) in (0..).zip(layout.groups.iter()) {
    for entry in entries.iter().filter(|entry| entry.stages.contains(stage)) {
      let mut target = msl::BindTarget {
        mutable: writable(entry.kind),
        ..Default::default()
      };
      match entry.kind {
        Binding::Uniform | Binding::DynamicUniform | Binding::Storage { .. } => {
          sizes |= matches!(entry.kind, Binding::Storage { .. });
          target.buffer = Some(buffers);
          buffers += 1;
        }
        Binding::Texture { .. } | Binding::StorageTexture { .. } => {
          target.texture = Some(textures);
          textures += 1;
        }
        Binding::Sampler { .. } => {
          target.sampler = Some(msl::BindSamplerTarget::Resource(samplers));
          samplers += 1;
        }
      }
      resources.resources.insert(
        naga::ResourceBinding {
          group,
          binding: entry.binding,
        },
        target,
      );
    }
  }
  if sizes {
    resources.sizes_buffer = Some(buffers);
  }
  resources
}

/// The vertex buffers as wgpu-hal maps them for vertex pulling: each in the
/// buffer slot counted down from the last.
pub(super) fn vertex_buffers(layout: &ShaderLayout) -> Vec<msl::VertexBufferMapping> {
  (0..)
    .zip(layout.vertex_buffers.iter())
    .map(|(index, buffer)| msl::VertexBufferMapping {
      id: MAX_BUFFERS - 1 - index,
      stride: u32::try_from(buffer.stride).expect("a vertex fits a u32"),
      step_mode: msl::VertexBufferStepMode::ByVertex,
      attributes: buffer
        .attributes
        .iter()
        .map(|attribute| msl::AttributeMapping {
          shader_location: attribute.location,
          offset: u32::try_from(attribute.offset).expect("an offset fits a u32"),
          format: match attribute.format {
            VertexFormat::Float32x2 => naga_types::VertexFormat::Float32x2,
            VertexFormat::Uint32 => naga_types::VertexFormat::Uint32,
          },
        })
        .collect(),
    })
    .collect()
}

/// Whether a resource of `kind` is written through, which Metal marks.
fn writable(kind: Binding) -> bool {
  matches!(
    kind,
    Binding::Storage { writable: true } | Binding::StorageTexture { .. }
  )
}
