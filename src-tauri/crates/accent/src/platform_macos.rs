// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ptr::NonNull;

use block2::RcBlock;
use objc2_app_kit::{NSColor, NSColorSpace, NSSystemColorsDidChangeNotification};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSString, NSUserDefaults};
use tauri::AppHandle;

use super::SystemAccent;

/// Absent exactly when the user's accent setting is Multicolour, which is
/// Apple's signal that an app should use its own accent colour instead.
const ACCENT_SETTING_KEY: &str = "AppleAccentColor";

pub(super) fn current() -> Option<SystemAccent> {
  let defaults = NSUserDefaults::standardUserDefaults();
  defaults.objectForKey(&NSString::from_str(ACCENT_SETTING_KEY))?;
  // The accent is a dynamic catalog colour; only an sRGB conversion has
  // readable components.
  let accent =
    NSColor::controlAccentColor().colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
  Some(SystemAccent {
    red: channel(accent.redComponent()),
    green: channel(accent.greenComponent()),
    blue: channel(accent.blueComponent()),
    tones: None,
  })
}

pub(super) fn initialize(app: &AppHandle, changed: fn(AppHandle)) {
  let app = app.clone();
  let block = RcBlock::new(move |_: NonNull<NSNotification>| changed(app.clone()));
  let center = NSNotificationCenter::defaultCenter();
  // The notification center retains the returned observer token for the
  // process lifetime. Screenwide installs this one observer exactly once.
  unsafe {
    center.addObserverForName_object_queue_usingBlock(
      Some(NSSystemColorsDidChangeNotification),
      None,
      None,
      &block,
    );
  }
}

fn channel(component: f64) -> u8 {
  (component.clamp(0.0, 1.0) * 255.0).round() as u8
}
