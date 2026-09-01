// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The macOS halves of the Glide commands. Each one is a thin shim onto the
//! module that owns the work, kept beside the event tap rather than inside it.

use objc2_app_kit::{
  NSHapticFeedbackManager, NSHapticFeedbackPattern, NSHapticFeedbackPerformanceTime,
  NSHapticFeedbackPerformer,
};
use tauri::AppHandle;

/// The platform half of the haptic command. AppKit only performs feedback from
/// the main thread, and the tick is fire and forget: waiting on the main thread
/// from a command would be a deadlock waiting to happen.
pub(in crate::glide) fn haptic(app: &AppHandle) -> Result<(), String> {
  app
    .run_on_main_thread(|| {
      NSHapticFeedbackManager::defaultPerformer().performFeedbackPattern_performanceTime(
        NSHapticFeedbackPattern::Generic,
        NSHapticFeedbackPerformanceTime::Now,
      );
    })
    .map_err(|error| error.to_string())
}
