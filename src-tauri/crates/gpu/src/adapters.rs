// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which graphics processor a device opens on. A laptop may have two: one
//! built into the processor, which draws the screen, and a faster discrete
//! one that switches off while nothing holds a device on it. Light work stays
//! on the integrated one, so the discrete one can sleep; heavy work goes to
//! the discrete one. A processor that cannot open the device gives way to the
//! next.
//!
//! macOS keeps one device for both: its Macs have one graphics processor, or
//! switch between two themselves.

/// What a device is opened for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Workload {
  /// Overlays, stills, references, the settings previews and recording:
  /// work any graphics processor keeps up with.
  Light,
  /// The editors, their playback and export, which want the fastest
  /// graphics processor.
  Heavy,
}

/// `adapters` in the order `workload` tries them.
pub(crate) fn ordered(mut adapters: Vec<wgpu::Adapter>, workload: Workload) -> Vec<wgpu::Adapter> {
  adapters.sort_by_cached_key(|adapter| rank(adapter, workload));
  adapters
}

fn rank(adapter: &wgpu::Adapter, workload: Workload) -> u8 {
  let kind = adapter.get_info().device_type;
  let fastest_first = match kind {
    wgpu::DeviceType::DiscreteGpu => 0,
    wgpu::DeviceType::IntegratedGpu => 1,
    wgpu::DeviceType::VirtualGpu | wgpu::DeviceType::Other => 2,
    wgpu::DeviceType::Cpu => 3,
  };
  match workload {
    Workload::Heavy => fastest_first,
    // Only while it draws a screen: on a desktop whose screens hang off the
    // discrete card, light work on the integrated one would have each frame
    // copied across to them.
    Workload::Light if kind == wgpu::DeviceType::IntegratedGpu && drives_a_display(adapter) => 0,
    Workload::Light => fastest_first + 1,
  }
}

#[cfg(target_os = "windows")]
fn drives_a_display(adapter: &wgpu::Adapter) -> bool {
  let Some(hal) = (unsafe { adapter.as_hal::<super::windows::raw_textures::Dx12>() }) else {
    return false;
  };
  unsafe { hal.raw_adapter().EnumOutputs(0) }.is_ok()
}

#[cfg(not(target_os = "windows"))]
fn drives_a_display(_adapter: &wgpu::Adapter) -> bool {
  false
}

/// The Direct3D adapter `adapter` is, to tell when two workloads would open
/// on the same one.
#[cfg(target_os = "windows")]
pub(crate) fn adapter_id(adapter: &wgpu::Adapter) -> Option<i64> {
  let hal = unsafe { adapter.as_hal::<super::windows::raw_textures::Dx12>() }?;
  let luid = unsafe { hal.raw_adapter().GetDesc1() }.ok()?.AdapterLuid;
  Some((i64::from(luid.HighPart) << 32) | i64::from(luid.LowPart))
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn adapter_id(_adapter: &wgpu::Adapter) -> Option<i64> {
  None
}
