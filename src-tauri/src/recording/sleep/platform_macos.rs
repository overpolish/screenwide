// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ptr::NonNull;

use block2::RcBlock;
use objc2_app_kit::{
  NSWorkspace, NSWorkspaceDidWakeNotification, NSWorkspaceWillSleepNotification,
};
use objc2_foundation::NSNotification;
use tauri::AppHandle;

/// Sleep and wake are posted on the workspace's own notification centre, not
/// the default one, on the main thread.
pub(super) fn initialize(app: &AppHandle, will_sleep: fn(&AppHandle), did_wake: fn(&AppHandle)) {
  let center = NSWorkspace::sharedWorkspace().notificationCenter();
  // SAFETY: AppKit defines both names for the life of the process.
  let observed = unsafe {
    [
      (NSWorkspaceWillSleepNotification, will_sleep),
      (NSWorkspaceDidWakeNotification, did_wake),
    ]
  };
  for (name, handler) in observed {
    let app = app.clone();
    let block = RcBlock::new(move |_: NonNull<NSNotification>| handler(&app));
    // The notification centre keeps each observer for the process lifetime.
    // Screenwide installs these two exactly once.
    unsafe {
      center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &block);
    }
  }
}
