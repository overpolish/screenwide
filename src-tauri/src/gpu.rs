// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The one wgpu device every renderer shares. Shaders are WGSL everywhere:
//! on macOS they run on Metal; on Windows on Direct3D 12, compiled through
//! the bundled DXC, because wgpu's default there, FXC, takes minutes on the
//! larger shaders. The canvas variants the build precompiled are the
//! exception on both.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use self::windows::bridge::{device_on_gpu_adapter, BridgedTexture, D3d11Bridge};
#[cfg(target_os = "windows")]
pub(crate) use self::windows::interop::{d3d11, D3d11Layer, SharedTexture};
#[cfg(target_os = "macos")]
pub(crate) mod macos;
pub(crate) mod surface;
mod textures;

use std::sync::{Arc, LazyLock};

pub(crate) struct Gpu {
  /// Surfaces are created from the instance that owns the device.
  pub(crate) instance: wgpu::Instance,
  pub(crate) device: wgpu::Device,
  pub(crate) queue: wgpu::Queue,
}

/// Opened on first use. A failure is kept, so every renderer reports the same
/// reason instead of retrying a device that cannot open.
static SHARED: LazyLock<Result<Gpu, String>> = LazyLock::new(|| pollster::block_on(Gpu::open()));

pub(crate) fn shared() -> Result<&'static Gpu, String> {
  SHARED.as_ref().map_err(Clone::clone)
}

impl Gpu {
  async fn open() -> Result<Self, String> {
    let instance = wgpu::Instance::new(instance_descriptor()?);
    // A dedicated GPU where there is one; wgpu falls back to the integrated
    // GPU on a machine without one.
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
      })
      .await
      .map_err(|error| format!("A graphics adapter is required: {error}"))?;
    let info = adapter.get_info();
    eprintln!("Graphics adapter: {} ({:?})", info.name, info.device_type);
    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor {
        label: Some("Screenwide"),
        // The canvas variants the build compiled are handed over as DXIL or
        // a Metal library; without it they are compiled from WGSL instead.
        required_features: if cfg!(any(target_os = "macos", target_os = "windows")) {
          adapter.features() & wgpu::Features::PASSTHROUGH_SHADERS
        } else {
          wgpu::Features::empty()
        },
        ..Default::default()
      })
      .await
      .map_err(|error| format!("The graphics device could not be opened: {error}"))?;
    // wgpu's default handler panics. A rendering mistake must cost a frame,
    // not the app.
    device.on_uncaptured_error(Arc::new(|error| {
      eprintln!("The graphics device reported an error: {error}");
    }));
    Ok(Self {
      instance,
      device,
      queue,
    })
  }
}

#[cfg(target_os = "windows")]
fn instance_descriptor() -> Result<wgpu::InstanceDescriptor, String> {
  let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
  // wgpu's DEBUG flag, on by default in debug builds, compiles every shader
  // with DXC's `-Od`: the preview shader then takes seconds to compile even
  // with no annotation in it. Validation stays on.
  descriptor.flags =
    wgpu::InstanceFlags::from_build_config().difference(wgpu::InstanceFlags::DEBUG);
  descriptor.backends = wgpu::Backends::DX12;
  descriptor.backend_options.dx12 = wgpu::Dx12BackendOptions {
    shader_compiler: wgpu::Dx12Compiler::DynamicDxc {
      dxc_path: self::windows::dxc::library_path()?,
    },
    // Overlay frames are drawn from the pointer's thread, which must never
    // wait on the compositor.
    latency_waitable_object: wgpu::Dx12UseFrameLatencyWaitableObject::None,
    ..Default::default()
  };
  Ok(descriptor)
}

#[cfg(target_os = "macos")]
fn instance_descriptor() -> Result<wgpu::InstanceDescriptor, String> {
  let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
  descriptor.backends = wgpu::Backends::METAL;
  Ok(descriptor)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn instance_descriptor() -> Result<wgpu::InstanceDescriptor, String> {
  Ok(wgpu::InstanceDescriptor::new_without_display_handle())
}
