// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared serialized state and outside-click semantics for native popovers.

use std::sync::{
  atomic::{AtomicBool, AtomicU64, Ordering},
  Mutex, MutexGuard,
};

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow};

use super::WindowLabel;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransientPopoverState {
  pub open: bool,
  pub revision: u64,
}

pub struct TransientPopover {
  lifecycle: Mutex<()>,
  open: AtomicBool,
  revision: AtomicU64,
}

impl TransientPopover {
  pub const fn new() -> Self {
    Self {
      lifecycle: Mutex::new(()),
      open: AtomicBool::new(false),
      revision: AtomicU64::new(0),
    }
  }

  pub fn is_open(&self) -> bool {
    self.open.load(Ordering::Relaxed)
  }

  pub fn lock(&self) -> MutexGuard<'_, ()> {
    self
      .lifecycle
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
  }

  pub fn revision(&self) -> u64 {
    self.revision.load(Ordering::Relaxed)
  }

  pub fn set_open(&self, open: bool) {
    self.open.store(open, Ordering::Relaxed);
    self.touch();
  }

  pub fn state(&self) -> TransientPopoverState {
    TransientPopoverState {
      open: self.is_open(),
      revision: self.revision(),
    }
  }

  pub fn touch(&self) {
    self.revision.fetch_add(1, Ordering::Relaxed);
  }

  pub fn should_dismiss(
    &self,
    app: &AppHandle,
    open_on_press: bool,
    inside_anchor: bool,
    x: f64,
    y: f64,
    owned_windows: &[WindowLabel],
  ) -> bool {
    open_on_press
      && self.is_open()
      && !inside_anchor
      && !owned_windows.iter().any(|label| {
        app
          .get_webview_window(label.as_str())
          .is_some_and(|window| coordinate_is_in_visible_window(x, y, &window))
      })
  }
}

fn coordinate_is_in_visible_window(x: f64, y: f64, window: &WebviewWindow) -> bool {
  if !window.is_visible().unwrap_or(false) {
    return false;
  }
  let Ok(position) = window.outer_position() else {
    return false;
  };
  let Ok(size) = window.outer_size() else {
    return false;
  };
  let Ok(scale) = window.scale_factor() else {
    return false;
  };
  let position = position.to_logical::<f64>(scale);
  let size = size.to_logical::<f64>(scale);

  x >= position.x
    && x <= position.x + size.width
    && y >= position.y
    && y <= position.y + size.height
}

#[cfg(test)]
mod tests {
  use super::TransientPopover;

  #[test]
  fn state_changes_are_revisioned() {
    let popover = TransientPopover::new();
    assert!(!popover.state().open);
    assert_eq!(popover.state().revision, 0);

    popover.set_open(true);
    assert!(popover.state().open);
    assert_eq!(popover.state().revision, 1);

    popover.touch();
    assert_eq!(popover.state().revision, 2);
  }
}
