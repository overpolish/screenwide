// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(
  dead_code,
  reason = "inspection methods support parity tests and future platform adapters"
)]

pub(crate) mod activity;
pub(crate) mod armed;
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

// Checkpoint 1 builds and verifies the shared policy before checkpoint 2 makes
// the native input path its runtime owner.
#[allow(unused_imports)]
pub(crate) use detector::{GlideDetection, GlideDetector, GlideSample};
#[allow(unused_imports)]
pub(crate) use folds::{GlideAction, GlideDetectorOptions};
pub(crate) use geometry::{
  corrected_origin, frame_fits, frame_fractions, frames_match, landing_point, GlideFrame,
};
#[allow(unused_imports)]
pub(crate) use regions::GlideRegion;
pub(crate) use runtime::{GlideEffects, GlideRuntime};
#[allow(unused_imports)]
pub(crate) use settling::GlidePhase;

#[cfg(test)]
mod tests;
