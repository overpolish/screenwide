// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;
use windows::Foundation::TypedEventHandler;
use windows::UI::ViewManagement::{UIColorType, UISettings};

use super::{AccentTones, SystemAccent};

/// Windows always exposes an accent colour; it has no Multicolour setting.
/// The tones are the ones WinUI's accent fills use: `AccentDark1` in light
/// appearance and `AccentLight2` in dark.
pub(super) fn current() -> Option<SystemAccent> {
  let settings = UISettings::new().ok()?;
  let read = |kind: UIColorType| {
    settings
      .GetColorValue(kind)
      .ok()
      .map(|color| [color.R, color.G, color.B])
  };
  let [red, green, blue] = read(UIColorType::Accent)?;
  let tones = match (
    read(UIColorType::AccentDark1),
    read(UIColorType::AccentLight2),
  ) {
    (Some(light), Some(dark)) => Some(AccentTones { light, dark }),
    _ => None,
  };
  Some(SystemAccent {
    red,
    green,
    blue,
    tones,
  })
}

pub(super) fn initialize(app: &AppHandle, changed: fn(AppHandle)) {
  let Ok(settings) = UISettings::new() else {
    return;
  };
  let app = app.clone();
  let handler = TypedEventHandler::new(move |_, _| {
    changed(app.clone());
    Ok(())
  });
  if settings.ColorValuesChanged(&handler).is_err() {
    return;
  }
  // The subscription lives only as long as the `UISettings` instance that owns
  // it. Screenwide installs this one observer exactly once, so the instance is
  // kept alive for the process lifetime rather than tracked.
  std::mem::forget(settings);
}
