// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas variants the build compiled (`build.rs`): the one with no
//! annotation, which a preview's first frame waits for, and the one with
//! every kind, which an editing preview draws with while any other set
//! compiles.

use super::canvas_modules::CanvasModules;
use super::canvas_variants::{every_kind, KindMask};

// `NO_ANNOTATIONS` and `EVERY_KIND`: `None` where the build compiled nothing
// for this target.
include!(concat!(env!("OUT_DIR"), "/precompiled_shaders.rs"));

/// The modules for `kinds` from what the build compiled, if it compiled them.
pub(super) fn modules(device: &wgpu::Device, kinds: KindMask) -> Option<CanvasModules> {
  let stages = if kinds == 0 {
    NO_ANNOTATIONS
  } else if kinds == every_kind() {
    EVERY_KIND
  } else {
    None
  };
  // SAFETY: the build compiled these stages from this crate's canvas shader
  // against the bindings `preview_bindings` lists, which the canvas layout is
  // built from; `precompiled_tests` draws with them and with the same shader
  // compiled from WGSL and requires the same pixels.
  let modules = unsafe {
    screenwide_gpu::precompiled_modules(device, stages, "Screenwide precompiled preview shader")
  }?;
  Some(CanvasModules {
    vertex: modules.vertex,
    fragment: modules.fragment,
  })
}
