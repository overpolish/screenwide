// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Animation curves for keyboard-key entrances, exits, and replacements.

use crate::exports::effect_animation::{damped_spring, ease_out_cubic};

const REPLACEMENT_EXIT_FRACTION: f32 = 0.6;
const REPLACEMENT_ENTER_DELAY: f32 = 0.3;

pub(super) fn ease_out(value: f32) -> f32 {
  ease_out_cubic(value)
}

pub(super) fn pop_spring(value: f32) -> f32 {
  damped_spring(value, 5.0, 6.0)
}

pub(super) fn replacement_exit_progress(value: f32) -> f32 {
  (value / REPLACEMENT_EXIT_FRACTION).clamp(0.0, 1.0)
}

pub(super) fn replacement_enter_progress(value: f32) -> f32 {
  ((value - REPLACEMENT_ENTER_DELAY) / (1.0 - REPLACEMENT_ENTER_DELAY)).clamp(0.0, 1.0)
}
