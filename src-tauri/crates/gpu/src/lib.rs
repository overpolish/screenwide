// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The wgpu device every renderer shares, open while something uses it
//! (`lifetime`). Shaders are WGSL everywhere: on macOS they run on Metal; on
//! Windows on Direct3D 12, compiled through the bundled DXC, because wgpu's
//! default there, FXC, takes minutes on the larger shaders. The canvas
//! variants the build precompiled are the exception on both.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::bridge::{
  device_on_gpu_adapter, video_device_on_gpu_adapter, BridgedTexture, D3d11Bridge,
};
#[cfg(target_os = "windows")]
pub use self::windows::interop::{d3d11, D3d11Layer, SharedTexture};
mod lifetime;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod mips;
mod precompiled;
pub mod surface;
mod textures;

pub use lifetime::{on_idle, open_soon, shared, DeviceCache};
pub use precompiled::{precompiled_modules, PrecompiledModules, PrecompiledStages};

use std::sync::Arc;

pub struct Gpu {
  /// Surfaces are created from the instance that owns the device.
  pub(crate) instance: wgpu::Instance,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  /// Which opening of the device this is, so a [`DeviceCache`] never hands
  /// out what was made on one closed since.
  generation: u64,
}

/// Frees what the device still holds for work that has ended, such as an
/// editor's picture and decoders after it closes. wgpu destroys dropped
/// resources only once it next submits or is polled, and the Direct3D 11
/// layer only once it next flushes; with nothing more to draw, neither would
/// happen and the memory would stay taken. Does nothing while the device is
/// closed.
pub fn release_unused() {
  let Some(gpu) = lifetime::current() else {
    return;
  };
  #[cfg(target_os = "windows")]
  windows::interop::release_unused();
  let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
}

impl Gpu {
  async fn open(generation: u64) -> Result<Self, String> {
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
        // wgpu's default reserves Direct3D 12 memory 128 to 256 MB at a time
        // for each kind of resource, sized for games. Idle, Screenwide holds a
        // few small textures, and on integrated graphics those empty blocks
        // are the computer's RAM. Large frames get allocations of their own
        // whichever is chosen. Metal ignores the hint.
        memory_hints: wgpu::MemoryHints::MemoryUsage,
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
      generation,
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
