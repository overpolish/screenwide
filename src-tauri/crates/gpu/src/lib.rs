// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The wgpu devices every renderer shares, one for each [`Workload`] on the
//! graphics processor that suits it, open while something uses it
//! (`lifetime`). Shaders are WGSL, run on Metal on macOS and on Direct3D 12
//! on Windows, and come compiled from the build ([`Shader`]); on Windows
//! what compiles at runtime goes through the bundled DXC, because wgpu's
//! default there, FXC, takes minutes on the larger shaders.

// The precompiled shaders' constants name this crate as others see it.
extern crate self as screenwide_gpu;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::bridge::{
  device_on_gpu_adapter, video_device_on_gpu_adapter, BridgedTexture, D3d11Bridge,
};
#[cfg(target_os = "windows")]
pub use self::windows::interop::{d3d11, D3d11Layer, SharedTexture};
mod adapters;
mod layouts;
mod lifetime;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod mips;
mod precompiled;
mod precompiled_shaders {
  include!(concat!(env!("OUT_DIR"), "/precompiled_shaders.rs"));
}
mod shader_layouts;
pub mod surface;
mod textures;

pub use adapters::Workload;
pub use layouts::bind_group_layout;
pub use lifetime::{on_idle, open_soon, shared, shared_for_readback, DeviceCache};
pub use precompiled::{
  compiling_wgsl, PrecompiledEntryPoint, PrecompiledShader, RenderStages, Shader,
};

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub struct Gpu {
  /// Surfaces are created from the instance that owns the device.
  pub(crate) instance: wgpu::Instance,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  /// Which opening of a device this is, so a [`DeviceCache`] never hands
  /// out what was made on another device, or on one closed since.
  generation: u64,
  /// The Direct3D adapter the device is on, where it is known.
  adapter_id: Option<i64>,
  /// How many drawing surfaces on the device are alive.
  live_surfaces: Arc<AtomicUsize>,
}

/// Frees what the open devices still hold for work that has ended, such as
/// an editor's picture and decoders after it closes. wgpu destroys dropped
/// resources only once it next submits or is polled, and the Direct3D 11
/// layer only once it next flushes; with nothing more to draw, neither would
/// happen and the memory would stay taken. Opens nothing.
pub fn release_unused() {
  for gpu in lifetime::open_devices() {
    #[cfg(target_os = "windows")]
    windows::interop::release_unused(&gpu);
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
  }
}

impl Gpu {
  /// The device for `workload` on the first graphics processor in its order
  /// that opens it, or none where that processor is `join`, the one another
  /// workload's device is already open on.
  async fn open(
    workload: Workload,
    generation: u64,
    join: Option<i64>,
  ) -> Result<Option<Self>, String> {
    let instance = wgpu::Instance::new(instance_descriptor()?);
    let adapters = adapters::ordered(
      instance.enumerate_adapters(wgpu::Backends::all()).await,
      workload,
    );
    if adapters.is_empty() {
      return Err("A graphics adapter is required".to_owned());
    }
    let mut failures = Vec::new();
    for adapter in adapters {
      let adapter_id = adapters::adapter_id(&adapter);
      if join.is_some() && adapter_id == join {
        return Ok(None);
      }
      let info = adapter.get_info();
      match request_device(&adapter).await {
        Ok((device, queue)) => {
          eprintln!(
            "Graphics adapter for {workload:?} work: {} ({:?})",
            info.name, info.device_type
          );
          return Ok(Some(Self {
            instance,
            device,
            queue,
            generation,
            adapter_id,
            live_surfaces: Arc::default(),
          }));
        }
        Err(error) => failures.push(format!("{}: {error}", info.name)),
      }
    }
    Err(format!(
      "The graphics device could not be opened: {}",
      failures.join("; ")
    ))
  }
}

async fn request_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), String> {
  let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor {
      label: Some("Screenwide"),
      // The canvas variants the build compiled are handed over as DXIL or a
      // Metal library; without it they are compiled from WGSL instead.
      required_features: if cfg!(any(target_os = "macos", target_os = "windows")) {
        adapter.features() & wgpu::Features::PASSTHROUGH_SHADERS
      } else {
        wgpu::Features::empty()
      },
      // wgpu's default reserves Direct3D 12 memory 128 to 256 MB at a time
      // for each kind of resource, sized for games. Idle, Screenwide holds a
      // few small textures, and on integrated graphics those empty blocks are
      // the computer's RAM. Large frames get allocations of their own
      // whichever is chosen. Metal ignores the hint.
      memory_hints: wgpu::MemoryHints::MemoryUsage,
      ..Default::default()
    })
    .await
    .map_err(|error| error.to_string())?;
  // wgpu's default handler panics. A rendering mistake must cost a frame,
  // not the app.
  device.on_uncaptured_error(Arc::new(|error| {
    eprintln!("The graphics device reported an error: {error}");
  }));
  Ok((device, queue))
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
