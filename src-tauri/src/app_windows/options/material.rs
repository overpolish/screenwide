// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a panel window's contents sit on. A panel opened from a window sits
//! on the system's material, as a native menu does; one opened from an
//! island is drawn in the island's black by its page, so the window carries
//! no material under it. The panel windows are reused, so each open says
//! which it wants.

use std::collections::BTreeSet;
use std::sync::Mutex;

use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::WebviewWindow;

/// The panels whose material has been taken away. A panel window starts
/// with the material its configuration gives it.
static WITHOUT_MATERIAL: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// The material every panel window is configured with (`tauri.conf.json`).
pub(crate) fn panel_material() -> WindowEffectsConfig {
  WindowEffectsConfig {
    color: None,
    effects: vec![Effect::UnderWindowBackground, Effect::Mica],
    radius: Some(10.0),
    state: Some(EffectState::Active),
  }
}

/// Puts the material under `panel`'s contents, or takes it away. Only a
/// change reaches the window: setting the same effects again rebuilds them.
/// The window's own shadow goes with the material: AppKit draws it with a
/// light rim round the window, which reads as a border on the island's
/// black, and the island has none.
pub(super) fn set(window: &WebviewWindow, panel: &str, material: bool) -> tauri::Result<()> {
  let mut without = WITHOUT_MATERIAL
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if without.contains(panel) != material {
    return Ok(());
  }
  if material {
    window.set_effects(panel_material())?;
  } else {
    clear_material(window)?;
  }
  window.set_shadow(material)?;
  if material {
    without.remove(panel);
  } else {
    without.insert(panel.to_owned());
  }
  Ok(())
}

/// Tauri's `set_effects(None)` does nothing on macOS, so the effect view it
/// put in is taken out directly, on the main thread AppKit asks for.
#[cfg(target_os = "macos")]
fn clear_material(window: &WebviewWindow) -> tauri::Result<()> {
  let target = window.clone();
  window.run_on_main_thread(move || {
    let _ = window_vibrancy::clear_vibrancy(&target);
  })
}

#[cfg(not(target_os = "macos"))]
fn clear_material(window: &WebviewWindow) -> tauri::Result<()> {
  window.set_effects(None)
}
