// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A shader's bind groups and vertex buffers, described once for both the
//! build, which compiles the shader against the GPU slots wgpu gives that
//! layout (`screenwide_wgsl::precompile`), and the renderer, which builds its
//! layouts from the same description (`screenwide_gpu::Shader`). Compiled
//! code is only valid against the layout it was compiled for, so neither side
//! writes the layout out on its own. It depends on nothing, so build scripts
//! take it as well.

use std::hash::{DefaultHasher, Hash, Hasher};

/// The stages that see a binding.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Stages(u8);

impl Stages {
  pub const VERTEX: Self = Self(1);
  pub const FRAGMENT: Self = Self(2);
  pub const VERTEX_FRAGMENT: Self = Self(3);
  pub const COMPUTE: Self = Self(4);

  pub const fn contains(self, other: Self) -> bool {
    self.0 & other.0 == other.0
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Binding {
  Uniform,
  /// A uniform read at an offset each draw gives.
  DynamicUniform,
  Storage {
    writable: bool,
  },
  Texture {
    sample: Sample,
    /// A 2D array rather than a 2D texture.
    array: bool,
  },
  /// A texture the shader writes and never reads.
  StorageTexture {
    format: StorageFormat,
  },
  Sampler {
    filtering: bool,
  },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Sample {
  Float,
  UnfilterableFloat,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum StorageFormat {
  Rgba8Unorm,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Entry {
  pub binding: u32,
  pub stages: Stages,
  pub kind: Binding,
}

/// Shorthand for the common case: an entry only the fragment stage sees.
pub const fn fragment(binding: u32, kind: Binding) -> Entry {
  Entry {
    binding,
    stages: Stages::FRAGMENT,
    kind,
  }
}

/// A filterable float 2D texture.
pub const TEXTURE: Binding = Binding::Texture {
  sample: Sample::Float,
  array: false,
};
/// A read-only storage buffer.
pub const READ_ONLY_STORAGE: Binding = Binding::Storage { writable: false };
pub const FILTERING_SAMPLER: Binding = Binding::Sampler { filtering: true };

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum VertexFormat {
  Float32x2,
  Uint32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Attribute {
  pub location: u32,
  pub offset: u64,
  pub format: VertexFormat,
}

/// A vertex buffer read once per vertex.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct VertexBuffer {
  pub stride: u64,
  pub attributes: &'static [Attribute],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ShaderLayout {
  /// Each bind group's entries, by group index, in ascending binding order:
  /// the order wgpu numbers them in.
  pub groups: &'static [&'static [Entry]],
  pub vertex_buffers: &'static [VertexBuffer],
}

impl ShaderLayout {
  /// What tells this layout from any other, so code compiled against one is
  /// never handed to a pipeline built from another. The build and the app
  /// are compiled by the same compiler, so both hash it alike.
  pub fn fingerprint(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.hash(&mut hasher);
    hasher.finish()
  }
}
