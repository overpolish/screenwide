// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ptr::NonNull;

use block2::RcBlock;
use objc2_app_kit::NSApplicationDidChangeScreenParametersNotification;
use objc2_foundation::{NSNotification, NSNotificationCenter};
use tauri::AppHandle;

/// macOS topology adapter. Window policy and reconciliation stay in the
/// platform-neutral parent module; this adapter only reports OS changes.
pub(super) fn initialize(app: &AppHandle, changed: fn(AppHandle)) {
  let app = app.clone();
  let block = RcBlock::new(move |_: NonNull<NSNotification>| changed(app.clone()));
  let center = NSNotificationCenter::defaultCenter();
  // The notification center retains the returned observer token for the
  // process lifetime. Screenwide installs this one observer exactly once.
  unsafe {
    center.addObserverForName_object_queue_usingBlock(
      Some(NSApplicationDidChangeScreenParametersNotification),
      None,
      None,
      &block,
    );
  }
}
