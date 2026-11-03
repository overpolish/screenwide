// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(
  dead_code,
  reason = "inspection methods support parity tests and future platform adapters"
)]

pub(crate) mod activity;
pub(crate) mod armed;
pub(crate) mod deliberate;
pub(crate) mod desktop_gesture;
pub(crate) mod desktop_selection;
pub(crate) mod desktops;
mod detector;
mod folds;
mod geometry;
mod intent;
mod monitor_intent;
pub(crate) mod monitors;
mod regions;
mod runtime;
mod settling;
pub(crate) mod taps;
pub(crate) mod trace;
mod travel;
pub(crate) mod tween;
pub(crate) mod tween_completion;
pub(crate) mod tween_motion;
pub(crate) mod tween_settle;
pub(crate) mod typing_guard;

pub(crate) use detector::{GlideDetection, GlideDetector, GlideSample};
pub(crate) use folds::{GlideAction, GlideDetectorOptions};
pub(crate) use geometry::{
  corrected_origin, frame_fits, frame_fractions, frames_match, grown_to_min, landing_point,
  GlideFrame,
};
pub(crate) use regions::GlideRegion;
pub(crate) use runtime::{GlideEffects, GlideRuntime};
#[cfg(test)]
pub(crate) use settling::GlidePhase;

#[cfg(test)]
mod tests;
