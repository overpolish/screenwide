// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Ruler {
  /// The zoom the projection uses. macOS clamped it at 1 in every projector,
  /// so a viewport can only ever magnify.
  pub(super) fn zoom(&self) -> f64 {
    self.viewport_zoom.max(1.0)
  }

  pub(crate) fn hover_progress(&self, now: Instant) -> f64 {
    if self.hovered_artifact_key == 0 {
      return 1.0;
    }
    (now
      .saturating_duration_since(self.hover_started)
      .as_secs_f64()
      / ANIMATION_DURATION.as_secs_f64())
    .clamp(0.0, 1.0)
  }

  pub(crate) fn hover_width(&self, now: Instant) -> f64 {
    HOVER_WIDTH_MIN + (HOVER_WIDTH_MAX - HOVER_WIDTH_MIN) * ease(self.hover_progress(now))
  }

  pub(crate) fn hover_alpha(&self) -> f64 {
    if self.hovered_artifact_key == 0 {
      0.0
    } else {
      HOVER_ALPHA * self.hover_opacity
    }
  }

  pub(crate) fn copied_amount(&self, now: Instant) -> f64 {
    self.copied.amount(now)
  }

  pub(crate) fn tolerance_amount(&self, now: Instant) -> f64 {
    self.tolerance.amount(now)
  }

  pub(crate) fn set_copied(&mut self, copied: bool, now: Instant) {
    self.copied.set(copied, false, now);
  }

  pub(crate) fn set_tolerance(&mut self, visible: bool, restart: bool, now: Instant) {
    self.tolerance.set(visible, restart, now);
  }

  pub(crate) fn set_hover(&mut self, key: u64, opacity: f64, started: Instant) {
    self.hovered_artifact_key = key;
    self.hover_opacity = opacity;
    self.hover_started = started;
  }

  pub(crate) fn hovered_artifact_key(&self) -> u64 {
    self.hovered_artifact_key
  }

  /// When the current hover pulse started, so an unchanged hover keeps easing
  /// from where it was instead of restarting every pointer sample.
  pub(crate) fn hover_started(&self) -> Instant {
    self.hover_started
  }

  /// Replaces this surface's datasets and reports which of the six redraw
  /// classes changed (`+ruler.m:993-1024`).
  pub(crate) fn replace_data(&mut self, next: &RulerData, viewport_changed: bool) -> DataChange {
    let change = DataChange {
      geometry: viewport_changed
        || !same(&self.data.measurements, &next.measurements)
        || !same(&self.data.probes, &next.probes)
        || !same(&self.data.guides, &next.guides)
        || !same(&self.data.guide_gaps, &next.guide_gaps)
        || !same(&self.data.radii, &next.radii)
        || !same(&self.data.centerlines, &next.centerlines)
        || !same(&self.data.inner_objects, &next.inner_objects),
      labels: viewport_changed
        || !same(
          &labelled_measurements(&self.data.measurements),
          &labelled_measurements(&next.measurements),
        )
        || !same(
          &labelled_probes(&self.data.probes),
          &labelled_probes(&next.probes),
        )
        || !same(
          &labelled_guide_gaps(&self.data.guide_gaps),
          &labelled_guide_gaps(&next.guide_gaps),
        )
        || !same(
          &labelled_radii(&self.data.radii),
          &labelled_radii(&next.radii),
        ),
    };
    self.data = next.clone();
    change
  }

  pub(crate) fn set_labels(&mut self, labels: Vec<LabelItem>) {
    self.labels = labels;
  }

  /// This surface's window onto the desktop plane, which is what decides which
  /// surface owns a label (`visible_world_rect`, `+ruler.m:1206-1213`).
  pub(crate) fn visible_world_rect(&self, offset: Point, bounds: Size) -> Rect {
    let zoom = self.zoom();
    Rect::from_xywh(
      offset.x + self.viewport_origin.x,
      offset.y + self.viewport_origin.y,
      bounds.width / zoom,
      bounds.height / zoom,
    )
  }

  /// The uv window the composited frozen snapshot is sampled through, so a
  /// zoomed viewport magnifies the desktop instead of the overlay
  /// (`screenshot_region_osc_macos.m:78-85`).
  pub(crate) fn snapshot_uv(&self, view: Size) -> Rect {
    let zoom = self.zoom();
    Rect::from_xywh(
      self.viewport_origin.x / view.width.max(1.0),
      self.viewport_origin.y / view.height.max(1.0),
      1.0 / zoom,
      1.0 / zoom,
    )
  }

  pub(crate) fn is_animating(&self, now: Instant) -> bool {
    self.visible
      && (self.copied.running(now)
        || self.tolerance.running(now)
        || (self.hovered_artifact_key != 0 && self.hover_progress(now) < 1.0))
  }

  /// Port of `screenwide_region_osc_ruler_vertex_capacity` (`:1196-1204`). The
  /// Windows builder grows a `Vec`, so this exists to keep the budget honest
  /// and to reserve in one step.
  pub(crate) fn vertex_capacity(&self) -> usize {
    let crosshair = usize::from(self.visible && self.crosshair) * 12;
    crosshair
      + self.data.measurements.len() * 48
      + self.data.probes.len() * 24
      + self.data.guides.len() * 12
      + self.data.guide_gaps.len() * 24
      + self.data.radii.len() * 12
      + self.data.centerlines.len() * 12
      + self.data.inner_objects.len() * 36
  }

  /// Port of `screenwide_region_osc_ruler_label_hit` (`:44-109`): measurement,
  /// probe, guide-gap then radius, first match wins.
  pub(crate) fn label_hit(&self, point: Point) -> Option<LabelHit> {
    if !self.visible {
      return None;
    }
    label_hit(&self.label_rects, point)
  }
}
