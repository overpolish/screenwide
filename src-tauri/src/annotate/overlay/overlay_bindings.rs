// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The bindings `annotate_overlay.wgsl` declares, all in group 0 and seen by
//! the fragment stage: the overlay's own uniform and pictures, then the
//! annotation lists and atlas at the bindings `annotations.wgsl` declares.
//! The renderer builds its layout from this list, and the build script reads
//! the same file to precompile the shader against the registers wgpu gives
//! that layout, so the two cannot drift apart. It depends on nothing else in
//! the crate for that reason.

#[derive(Clone, Copy)]
pub(crate) enum OverlayBinding {
  Uniform,
  /// A read-only storage buffer.
  Storage,
  /// A filterable float 2D texture.
  Texture,
  /// A filtering sampler.
  Sampler,
}

pub(crate) const OVERLAY_BINDINGS: [(u32, OverlayBinding); 10] = {
  use OverlayBinding::*;
  [
    (0, Uniform),
    (1, Texture),
    (2, Texture),
    (3, Sampler),
    (4, Storage),
    (7, Storage),
    (8, Storage),
    (9, Texture),
    (10, Storage),
    (11, Storage),
  ]
};
