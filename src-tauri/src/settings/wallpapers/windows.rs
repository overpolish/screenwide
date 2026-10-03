// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures on the Windows desktop.
//!
//! `IDesktopWallpaper` names each monitor's picture. A slideshow or Windows
//! Spotlight can answer with nothing, or with a file that has since gone, and
//! then the copy Windows scales for the desktop, `TranscodedWallpaper`, is
//! what is on screen.

use std::path::PathBuf;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::Com::{
  CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Shell::{DesktopWallpaper, IDesktopWallpaper};

/// The scaled copy, under the roaming application data folder. It has no
/// extension; the decoder reads its format from its contents.
const TRANSCODED: &str = r"Microsoft\Windows\Themes\TranscodedWallpaper";

/// COM for the calling thread, tolerant of a thread that already holds it in
/// another apartment.
struct Com {
  owned: bool,
}

impl Com {
  fn enter() -> Self {
    let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    Self {
      owned: result.is_ok(),
    }
  }
}

impl Drop for Com {
  fn drop(&mut self) {
    if self.owned {
      unsafe { CoUninitialize() };
    }
  }
}

/// A string COM allocated for the caller, read and then freed.
fn take(text: PWSTR) -> Option<String> {
  let value = unsafe { text.to_string() }.ok();
  unsafe { CoTaskMemFree(Some(text.0 as _)) };
  value.filter(|value| !value.is_empty())
}

fn transcoded() -> Option<PathBuf> {
  let path = PathBuf::from(std::env::var_os("APPDATA")?).join(TRANSCODED);
  path.is_file().then_some(path)
}

/// Each monitor's picture, in the order Windows lists the monitors.
fn per_monitor() -> Vec<Option<PathBuf>> {
  let _com = Com::enter();
  let Ok(wallpaper) =
    (unsafe { CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL) })
  else {
    return Vec::new();
  };
  let count = unsafe { wallpaper.GetMonitorDevicePathCount() }.unwrap_or(0);
  (0..count)
    .filter_map(|index| {
      let monitor = unsafe { wallpaper.GetMonitorDevicePathAt(index) }.ok()?;
      let picture = unsafe { wallpaper.GetWallpaper(PCWSTR(monitor.0)) };
      unsafe { CoTaskMemFree(Some(monitor.0 as _)) };
      Some(picture.ok().and_then(take).map(PathBuf::from))
    })
    .collect()
}

pub(super) fn active_pictures() -> Vec<PathBuf> {
  let monitors = per_monitor();
  if monitors.is_empty() {
    return transcoded().into_iter().collect();
  }
  monitors
    .into_iter()
    .filter_map(|picture| picture.filter(|path| path.is_file()).or_else(transcoded))
    .collect()
}
