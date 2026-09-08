// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;
use windows::Foundation::TypedEventHandler;
use windows::UI::ViewManagement::{UIColorType, UISettings};

use super::SystemAccent;

/// Windows always exposes an accent colour; it has no Multicolour setting.
pub(super) fn current() -> Option<SystemAccent> {
  let color = UISettings::new()
    .ok()?
    .GetColorValue(UIColorType::Accent)
    .ok()?;
  Some(SystemAccent {
    red: color.R,
    green: color.G,
    blue: color.B,
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
