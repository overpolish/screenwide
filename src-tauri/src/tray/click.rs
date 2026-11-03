// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a click on the tray icon asks for. A click toggles the recording
//! controls, as their shortcut does. The menu opens on a right click, which
//! the tray does itself, and on a Control-click on macOS, where that is a
//! right click too but reaches the icon as a left one.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClickAction {
  ShowMenu,
  ToggleRecordingControls,
}

/// Remembers, between the icon's events, a release that must not toggle.
#[derive(Default)]
pub(super) struct ClickFilter {
  skip_next_release: AtomicBool,
}

impl ClickFilter {
  /// Toggles on the left button's release, as a click is read everywhere
  /// else. Windows reports a double click as press, release, double click,
  /// release, and the second release would put the controls straight back,
  /// so it is skipped. A Control-click opens the menu on its press, as the
  /// right button does, and its release is skipped; a release that never
  /// comes, swallowed by the menu, is forgotten at the next press.
  pub(super) fn action(&self, event: &TrayIconEvent, control_held: bool) -> Option<ClickAction> {
    match event {
      TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Down,
        ..
      } => {
        self
          .skip_next_release
          .store(control_held, Ordering::Relaxed);
        control_held.then_some(ClickAction::ShowMenu)
      }
      TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } => (!self.skip_next_release.swap(false, Ordering::Relaxed))
        .then_some(ClickAction::ToggleRecordingControls),
      TrayIconEvent::DoubleClick {
        button: MouseButton::Left,
        ..
      } => {
        self.skip_next_release.store(true, Ordering::Relaxed);
        None
      }
      _ => None,
    }
  }
}

/// Whether Control is held: only on macOS does it turn a click into a right
/// click.
pub(super) fn control_held() -> bool {
  #[cfg(target_os = "macos")]
  {
    objc2_app_kit::NSEvent::modifierFlags_class()
      .contains(objc2_app_kit::NSEventModifierFlags::Control)
  }
  #[cfg(not(target_os = "macos"))]
  {
    false
  }
}

#[cfg(test)]
mod tests;
