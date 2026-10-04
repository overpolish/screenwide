// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas variants the build compiled (`build/precompiled_shaders.rs`):
//! the one with no annotation, which a preview's first frame waits for, and
//! the one with every kind, which an editing preview draws with while any
//! other set compiles. They are DXIL on Windows and Metal libraries on macOS,
//! handed to the device as they are, so only the GPU driver's own pass is
//! left. A build for any other platform has none, and its variants compile
//! from WGSL.

use std::borrow::Cow;

use super::canvas_modules::CanvasModules;
use super::canvas_variants::{every_kind, KindMask};

/// A variant's stages as the build compiled them.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
struct Stages {
  vertex: &'static [u8],
  fragment: &'static [u8],
}

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
  }?;
  if !device
    .features()
    .contains(wgpu::Features::PASSTHROUGH_SHADERS)
  {
    return None;
  }
  #[cfg_attr(
    not(any(target_os = "macos", target_os = "windows")),
    allow(unused_variables)
  )]
  let module = |bytes: &'static [u8], entry: &'static str| {
    let descriptor = wgpu::ShaderModuleDescriptorPassthrough {
      label: Some("Screenwide precompiled preview shader"),
      entry_points: Cow::Owned(vec![wgpu::PassthroughShaderEntryPoint {
        name: entry.into(),
        workgroup_size: (0, 0, 0),
      }]),
      #[cfg(target_os = "windows")]
      dxil: Some(Cow::Borrowed(bytes)),
      #[cfg(target_os = "macos")]
      metallib: Some(Cow::Borrowed(bytes)),
      ..Default::default()
    };
    // SAFETY: the build compiled these bytes from this crate's canvas shader
    // against the bindings `preview_bindings` lists, laid out the way wgpu 30
    // lays out that bind group; `precompiled_tests` draws with them and with
    // the same shader compiled from WGSL and requires the same pixels. The
    // Metal libraries leave buffer accesses unchecked, as wgpu cannot give
    // them buffer lengths (`build/precompiled_shaders/metallib.rs`).
    unsafe { device.create_shader_module_passthrough(descriptor) }
  };
  Some(CanvasModules {
    vertex: module(stages.vertex, "vs_main"),
    fragment: module(stages.fragment, "fs_main"),
  })
}
