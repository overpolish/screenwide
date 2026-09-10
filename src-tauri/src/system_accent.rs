// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::RwLock;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "macos")]
#[path = "system_accent/platform_macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "system_accent/platform_windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "system_accent/platform_other.rs"]
mod platform;

/// Emitted to every window whenever the operating system accent changes. The
/// payload is the new accent, or `null` when the app should use its own.
const ACCENT_CHANGED_EVENT: &str = "system-accent-changed";

/// The brand accent, used wherever the platform has no accent of its own.
/// Matches `--color-primary`'s fallback in `src/index.css`.
const BRAND_ACCENT: [u8; 3] = [0xd8, 0x1b, 0x60];

/// The operating system accent colour, in sRGB.
#[derive(Clone, Copy, Serialize)]
pub struct SystemAccent {
  red: u8,
  green: u8,
  blue: u8,
}

impl SystemAccent {
  /// The accent as sRGB components in 0..1, the form every renderer wants.
  pub fn components(self) -> [f32; 3] {
    [
      f32::from(self.red) / 255.0,
      f32::from(self.green) / 255.0,
      f32::from(self.blue) / 255.0,
    ]
  }
}

/// The last accent read from the platform. Native renderers ask for the
/// accent once per control per frame, and both platform reads touch UI
/// frameworks, so the value is read when it changes rather than when it is
/// used. `None` means nothing has been read yet.
static CACHE: RwLock<Option<Option<SystemAccent>>> = RwLock::new(None);

/// Reads the current accent. `None` means the platform has no accent to
/// follow, so the app keeps its own brand colour.
pub fn current() -> Option<SystemAccent> {
  if let Ok(cache) = CACHE.read() {
    if let Some(accent) = *cache {
      return accent;
    }
  }
  refresh()
}

fn refresh() -> Option<SystemAccent> {
  let accent = platform::current();
  if let Ok(mut cache) = CACHE.write() {
    *cache = Some(accent);
  }
  accent
}

/// Reads the current accent. `None` means the platform has no accent to
/// follow, so the app keeps its own brand colour.
#[tauri::command]
pub fn get_system_accent() -> Option<SystemAccent> {
  current()
}

/// The accent every native surface paints with: the operating system's when
/// the user has chosen one, the brand colour otherwise. This is the Rust twin
/// of `--color-primary`, so native chrome and the web UI never disagree.
pub fn accent_rgb() -> [f32; 3] {
  current().map_or_else(
    || {
      [
        f32::from(BRAND_ACCENT[0]) / 255.0,
        f32::from(BRAND_ACCENT[1]) / 255.0,
        f32::from(BRAND_ACCENT[2]) / 255.0,
      ]
    },
    SystemAccent::components,
  )
}

/// Drops the cached accent so the next read goes back to the platform.
/// Native observers of the platform's accent notification call this before
/// they redraw, because the notification reaches every observer in an
/// unspecified order.
#[no_mangle]
pub extern "C" fn screenwide_system_accent_invalidate() {
  if let Ok(mut cache) = CACHE.write() {
    *cache = None;
  }
}

/// Writes the operating system accent as sRGB rgba in 0..1 and returns 1.
/// Returns 0, leaving `out` untouched, when the platform has no accent (macOS
/// Multicolour, or a platform without the concept), which is the caller's
/// signal to use the brand colour.
///
/// # Safety
/// `out` must be null or point to a writable `[f32; 4]`.
#[no_mangle]
pub unsafe extern "C" fn screenwide_system_accent(out: *mut [f32; 4]) -> u32 {
  let Some(accent) = current() else {
    return 0;
  };
  let Some(out) = out.as_mut() else {
    return 0;
  };
  let [red, green, blue] = accent.components();
  *out = [red, green, blue, 1.0];
  1
}

/// Installs the platform observer once, at startup.
pub fn initialize(app: &AppHandle) {
  platform::initialize(app, accent_changed);
}

fn accent_changed(app: AppHandle) {
  let _ = app.emit(ACCENT_CHANGED_EVENT, refresh());
}
