// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared serialized state and outside-click semantics for native popovers.

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex, MutexGuard,
};

use tauri::WebviewWindow;

pub struct TransientPopover {
  lifecycle: Mutex<()>,
  open: AtomicBool,
}

impl TransientPopover {
  pub const fn new() -> Self {
    Self {
      lifecycle: Mutex::new(()),
      open: AtomicBool::new(false),
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

  pub fn set_open(&self, open: bool) {
    self.open.store(open, Ordering::Relaxed);
  }
}

pub(super) fn coordinate_is_in_visible_window(x: f64, y: f64, window: &WebviewWindow) -> bool {
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
