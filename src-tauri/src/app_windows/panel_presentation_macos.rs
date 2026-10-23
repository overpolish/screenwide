// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use objc2_app_kit::{NSWindow, NSWindowAnimationBehavior};

/// Every window this app floats over the desktop is present or absent, never
/// animated as though it were a conventional utility window, as DWM's
/// transitions are turned off for them on Windows. AppKit's own animation
/// scales a window from its centre: an overlay on each display would zoom in
/// one by one, and the island, which animates itself out of the notch and back
/// in, would have its top edge pulled off the screen's.
pub(crate) fn disable_order_animation(window: &NSWindow) {
  window.setAnimationBehavior(NSWindowAnimationBehavior::None);
}
