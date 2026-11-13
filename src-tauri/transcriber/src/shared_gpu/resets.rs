// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Models that have reset the GPU, remembered in a file beside each with the
//! graphics drivers there were when it did, so later jobs leave the GPU alone
//! until a driver changes, which may be what mends it. Only Windows resets a
//! GPU for being held too long, so elsewhere nothing is remembered.

use std::path::{Path, PathBuf};

fn note(model: &Path) -> PathBuf {
  let mut name = model.file_name().unwrap_or_default().to_owned();
  name.push(".gpu-reset");
  model.with_file_name(name)
}

/// Whether the model at `model` has reset the GPU under the drivers there
/// are now.
pub(super) fn reset_before(model: &Path) -> bool {
  #[cfg(windows)]
  return std::fs::read_to_string(note(model)).is_ok_and(|kept| kept == drivers());
  #[cfg(not(windows))]
  {
    let _ = note(model);
    false
  }
}

/// Remembers that the model at `model` reset the GPU.
pub(super) fn remember(model: &Path) {
  #[cfg(windows)]
  if let Err(error) = std::fs::write(note(model), drivers()) {
    eprintln!("Could not remember the GPU reset: {error}");
  }
  #[cfg(not(windows))]
  let _ = model;
}

/// Every graphics adapter's name and driver version, a line each.
#[cfg(windows)]
fn drivers() -> String {
  use windows::core::Interface;
  use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIDevice, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
  };

  let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
    return String::new();
  };
  let mut lines = Vec::new();
  let mut index = 0;
  while let Ok(adapter) = unsafe { factory.EnumAdapters1(index) } {
    index += 1;
    let Ok(description) = (unsafe { adapter.GetDesc1() }) else {
      continue;
    };
    if description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
      continue;
    }
    let name = String::from_utf16_lossy(&description.Description);
    let version = unsafe { adapter.CheckInterfaceSupport(&IDXGIDevice::IID) }.unwrap_or(0);
    lines.push(format!(
      "{} {}.{}.{}.{}",
      name.trim_end_matches('\0'),
      version >> 48,
      (version >> 32) & 0xffff,
      (version >> 16) & 0xffff,
      version & 0xffff
    ));
  }
  lines.sort();
  lines.dedup();
  lines.join("\n")
}
