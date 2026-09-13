// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "evaluation/ranges.rs"]
mod ranges;

use super::*;

impl KeyboardCompositor {
  pub(crate) fn evaluate_fitted_with_timeline(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
    dimensions: (u32, u32),
    timeline: Option<&crate::editor::timeline_edit::TimelinePlan>,
  ) -> Option<KeyboardOverlay> {
    self.evaluate_fitted_with_ranges(
      position_ms,
      settings,
      dimensions,
      timeline.map(|timeline| timeline.ranges()),
    )
  }

  pub(crate) fn evaluate_fitted_with_ranges(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
    dimensions: (u32, u32),
    ranges: Option<&[crate::editor::timeline_edit::TimelineRange]>,
  ) -> Option<KeyboardOverlay> {
    let mut overlay = self.evaluate_with_ranges(position_ms, settings, ranges)?;
    overlay.scale = overlay
      .requested_scale
      .min((self.maximum_size_percent(dimensions.0, dimensions.1) / 100.0) as f32);
    Some(overlay)
  }

  #[cfg(test)]
  pub(crate) fn evaluate(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
  ) -> Option<KeyboardOverlay> {
    self.evaluate_with_timeline(position_ms, settings, None)
  }

  #[cfg(test)]
  pub(crate) fn evaluate_with_timeline(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
    timeline: Option<&crate::editor::timeline_edit::TimelinePlan>,
  ) -> Option<KeyboardOverlay> {
    self.evaluate_with_ranges(
      position_ms,
      settings,
      timeline.map(|timeline| timeline.ranges()),
    )
  }
}
