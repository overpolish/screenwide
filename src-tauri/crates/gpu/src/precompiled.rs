// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shader stages the build compiled (`screenwide_wgsl::precompile`): DXIL on
//! Windows and Metal libraries on macOS, handed to the device as they are,
//! so only the GPU driver's own pass is left. A build for any other platform
//! has none, and its shaders compile from WGSL.

use std::borrow::Cow;

/// A shader's `vs_main` and `fs_main` as the build compiled them.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
#[derive(Clone, Copy)]
pub struct PrecompiledStages {
  pub vertex: &'static [u8],
  pub fragment: &'static [u8],
}

pub struct PrecompiledModules {
  pub vertex: wgpu::ShaderModule,
  pub fragment: wgpu::ShaderModule,
}

/// The modules for `stages`, if the build compiled them and the device takes
/// compiled shaders.
///
/// # Safety
///
/// `stages` must be what the build compiled from the shader the pipeline is
/// made for, against the bindings its layout is built from: the device
/// cannot check compiled code against a layout. The Metal libraries leave
/// buffer accesses unchecked, as wgpu cannot give them buffer lengths.
pub unsafe fn precompiled_modules(
  device: &wgpu::Device,
  stages: Option<PrecompiledStages>,
  label: &'static str,
) -> Option<PrecompiledModules> {
  let stages = stages?;
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
      label: Some(label),
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
    // SAFETY: the caller vouches for the bytes.
    unsafe { device.create_shader_module_passthrough(descriptor) }
  };
  Some(PrecompiledModules {
    vertex: module(stages.vertex, "vs_main"),
    fragment: module(stages.fragment, "fs_main"),
  })
}
