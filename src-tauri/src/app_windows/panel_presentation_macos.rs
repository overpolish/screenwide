// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use objc2_app_kit::{NSPanel, NSWindowAnimationBehavior};

/// Full-monitor Region chrome should be present or absent, never animated as
/// though it were a conventional utility window.
pub fn configure_order_animation(label: &str, panel: &NSPanel) {
  if label == "region-selector" {
    panel.setAnimationBehavior(NSWindowAnimationBehavior::None);
  }
}
