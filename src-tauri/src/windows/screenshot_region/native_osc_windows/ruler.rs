// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Ruler overlay: crosshair, probes, guides, guide gaps, corner-radius
//! arcs, centerlines, inner objects, measurement boxes, the pooled floating
//! labels and the cursor-readout loupe. Port of
//! `screenshot_region_osc_macos+ruler.m`.
//!
//! macOS gave the loupe and every label its own `NSVisualEffectView` with a
//! small `CAMetalLayer` on top, and hit-tested labels against those AppKit
//! frames. Here all of it folds into the surface's one swap chain the way the
//! OCR chrome does (plan decision 10): a label is a run of quads plus a
//! [`Segment`] carrying its own fill, foreground and label texture, and the
//! rectangles this file lays out are also the only hit-test source there is.
//!
//! All analysis, gestures, viewports, artifacts and history stay in the
//! portable `crate::ruler` document; this file pulls its draw data, diffs it,
//! projects it through the per-display viewport and turns it into vertices.

use std::time::{Duration, Instant};

use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11ShaderResourceView};

use super::ocr::Segment;
use super::renderer::{self, Vertex};
use super::text::TextCache;
use crate::osc::{
  controls::{
    control_metrics, control_visual, Appearance, ControlColor, ControlKind, ControlMetrics,
    ControlSize, ControlStyle, Interaction,
  },
  geometry::{Point, Rect, Size},
};
use crate::ruler::render::{
  CenterlinePacket, GuideGapPacket, GuidePacket, InnerObjectPacket, MeasurementPacket, ProbePacket,
  RadiusPacket, ViewportPacket,
};

/// Every ruler transition — the copied checkmark, the hover pulse and the
/// tolerance notice — runs over this window (`+ruler.m:7-9`).
pub(crate) const ANIMATION_DURATION: Duration = Duration::from_millis(160);
/// How long a copied checkmark or a tolerance notice stays up before it fades
/// back out (`+ruler.m:1044-1070`).
pub(crate) const EXPIRY: Duration = Duration::from_millis(900);
/// The hover halo's alpha; only its width animates (`ruler_hover_alpha`).
const HOVER_ALPHA: f64 = 0.24;
const HOVER_WIDTH_MIN: f64 = 3.0;
const HOVER_WIDTH_MAX: f64 = 8.0;

/// `screenwide_osc_control_spacing`, reached through the `#[no_mangle]` export
/// because `osc::controls::style` is a private module of the frozen portable
/// tree — the same route the icon atlas takes in `surface.rs`.
#[repr(C)]
#[derive(Clone, Copy)]
struct NativeControlSpacing {
  tight: f64,
  control: f64,
  control_inset: f64,
  section: f64,
  window_inset: f64,
}

extern "C" {
  fn screenwide_osc_control_spacing() -> NativeControlSpacing;
}

fn spacing() -> (f64, f64) {
  let value = unsafe { screenwide_osc_control_spacing() };
  (value.control, value.control_inset)
}

fn metrics() -> ControlMetrics {
  control_metrics(ControlKind::Button, ControlSize::Compact)
}

/// The eight datasets a ruler-flagged result pulls from the document. macOS
/// stored each as an `NSData` blob per surface; the Rust twin keeps the packet
/// vectors and compares their bytes, which is the same comparison.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RulerData {
  pub measurements: Vec<MeasurementPacket>,
  pub viewports: Vec<ViewportPacket>,
  pub probes: Vec<ProbePacket>,
  pub guides: Vec<GuidePacket>,
  pub guide_gaps: Vec<GuideGapPacket>,
  pub radii: Vec<RadiusPacket>,
  pub centerlines: Vec<CenterlinePacket>,
  pub inner_objects: Vec<InnerObjectPacket>,
}

/// One label's hit rectangle in surface-local logical points. macOS hit-tested
/// the material surfaces' AppKit frames; with the chrome folded in there are no
/// frames, so this list is the source of truth (`+ruler.m:44-109`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LabelRect {
  pub id: u64,
  /// `1 measurement, 2 probe, 3 guide gap, 4 radius`.
  pub kind: u8,
  pub rect: Rect,
}

/// What a label hit reports back: the artifact plus the label's centre, which
/// the drag gesture needs in desktop coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LabelHit {
  pub id: u64,
  pub kind: u8,
  pub center: Point,
}

/// One label this surface owns. Guide gaps and radii are laid out through the
/// probe path exactly as `render_probe_label` did, so they travel as the
/// derived probe plus, for a radius, the packet its text comes from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LabelItem {
  Measurement(MeasurementPacket),
  Probe(ProbePacket),
  GuideGap(ProbePacket),
  Radius(RadiusPacket),
}

/// A three-part eased transition: where it came from, where it is going and
/// when it started (`animation_amount`, `+ruler.m:180-197`).
#[derive(Clone, Copy, Debug)]
struct Animation {
  from: f64,
  target: bool,
  started: Instant,
}

/// An instant far enough in the past that every transition reads as settled.
fn settled() -> Instant {
  Instant::now()
    .checked_sub(ANIMATION_DURATION)
    .unwrap_or_else(Instant::now)
}

impl Default for Animation {
  fn default() -> Self {
    Self {
      from: 0.0,
      target: false,
      started: settled(),
    }
  }
}

impl Animation {
  fn amount(&self, now: Instant) -> f64 {
    let elapsed = now.saturating_duration_since(self.started).as_secs_f64();
    let progress = (elapsed / ANIMATION_DURATION.as_secs_f64()).clamp(0.0, 1.0);
    let eased = ease(progress);
    let target = if self.target { 1.0 } else { 0.0 };
    self.from + (target - self.from) * eased
  }

  fn running(&self, now: Instant) -> bool {
    now.saturating_duration_since(self.started) < ANIMATION_DURATION
  }

  /// `restart` replays from zero even when the target is unchanged, which is
  /// how a tolerance notice re-triggers for a new mode (`set_tolerance_visible`).
  fn set(&mut self, target: bool, restart: bool, now: Instant) {
    if !restart && self.target == target {
      return;
    }
    self.from = if restart { 0.0 } else { self.amount(now) };
    self.target = target;
    self.started = now;
  }
}

fn ease(progress: f64) -> f64 {
  1.0 - (1.0 - progress).powi(3)
}

/// Per-surface ruler state. The values above the datasets are mirrored from the
/// root on every result; the datasets are this surface's own copy so change
/// detection can be per surface, as it was on macOS.
pub(crate) struct Ruler {
  pub visible: bool,
  pub crosshair: bool,
  pub interaction_active: bool,
  pub transient_chrome: bool,
  /// The pointer in this surface's own logical points.
  pub point: Point,
  pub color: u32,
  pub tolerance_visible: bool,
  pub tolerance_mode: u8,
  hovered_artifact_key: u64,
  hover_opacity: f64,
  hover_started: Instant,
  pub viewport_zoom: f64,
  pub viewport_origin: Point,
  /// The desktop union, which fixes the label width so numbers never resize
  /// the plate (`reserved_dimensions_length`).
  pub desktop_size: Size,
  data: RulerData,
  labels: Vec<LabelItem>,
  label_rects: Vec<LabelRect>,
  copied: Animation,
  tolerance: Animation,
  text: TextCache,
}

impl Default for Ruler {
  fn default() -> Self {
    Self {
      visible: false,
      crosshair: false,
      interaction_active: false,
      transient_chrome: true,
      point: Point::default(),
      color: 0,
      tolerance_visible: false,
      tolerance_mode: 0,
      hovered_artifact_key: 0,
      hover_opacity: 0.0,
      hover_started: settled(),
      viewport_zoom: 1.0,
      viewport_origin: Point::default(),
      desktop_size: Size::default(),
      data: RulerData::default(),
      labels: Vec::new(),
      label_rects: Vec::new(),
      copied: Animation::default(),
      tolerance: Animation::default(),
      text: TextCache::default(),
    }
  }
}

impl Ruler {
  /// The zoom the projection uses. macOS clamped it at 1 in every projector,
  /// so a viewport can only ever magnify.
  fn zoom(&self) -> f64 {
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

  /// Port of `screenwide_region_osc_ruler_add_vertices` (`:1797-2015`), in its
  /// exact order: crosshair, probes, guides, gaps, radii, centerlines, inner
  /// objects, measurements.
  pub(crate) fn add_world_vertices(
    &self,
    out: &mut Vec<Vertex>,
    view: Size,
    scale: f64,
    display_id: u32,
    offset: Point,
    now: Instant,
  ) {
    if !self.visible {
      return;
    }
    out.reserve(self.vertex_capacity());
    let (tick, _) = spacing();
    let hover_width = self.hover_width(now);
    let zoom = self.zoom();

    if self.crosshair {
      let x = renderer::snap(self.point.x, scale);
      let y = renderer::snap(self.point.y, scale);
      if x >= 0.0 && x <= view.width {
        renderer::add_line(
          out,
          view,
          Point { x, y: 0.0 },
          Point { x, y: view.height },
          1.0 / scale,
          28,
        );
      }
      if y >= 0.0 && y <= view.height {
        renderer::add_line(
          out,
          view,
          Point { x: 0.0, y },
          Point { x: view.width, y },
          1.0 / scale,
          28,
        );
      }
    }

    for probe in &self.data.probes {
      let live = probe.flags & 4 != 0;
      if live && (!self.transient_chrome || probe.display_id != display_id) {
        continue;
      }
      let (start, end, position) = self.project_probe(*probe, offset);
      let (start, end) = if start > end {
        (end, start)
      } else {
        (start, end)
      };
      let start = renderer::snap(start, scale);
      let end = renderer::snap(end, scale);
      let position = renderer::snap(position, scale);
      let along = |value: f64| axis_point(probe.axis, value, position);
      if probe.padding[0] != 0 {
        renderer::add_line(out, view, along(start), along(end), hover_width, 32);
      }
      renderer::add_line(out, view, along(start), along(end), 1.0 / scale, 28);
      for edge in [start, end] {
        renderer::add_line(
          out,
          view,
          axis_point(probe.axis, edge, position - tick),
          axis_point(probe.axis, edge, position + tick),
          1.0 / scale,
          28,
        );
      }
    }

    for guide in &self.data.guides {
      if guide.display_id != display_id {
        continue;
      }
      if guide.axis == 1 {
        let x = renderer::snap(
          (guide.position - offset.x - self.viewport_origin.x) * zoom,
          scale,
        );
        if x >= 0.0 && x <= view.width {
          let (from, to) = (Point { x, y: 0.0 }, Point { x, y: view.height });
          if guide.padding[0] != 0 {
            renderer::add_line(out, view, from, to, hover_width, 38);
          }
          renderer::add_line(out, view, from, to, 1.0 / scale, 36);
        }
      } else if guide.axis == 2 {
        let y = renderer::snap(
          (guide.position - offset.y - self.viewport_origin.y) * zoom,
          scale,
        );
        if y >= 0.0 && y <= view.height {
          let (from, to) = (Point { x: 0.0, y }, Point { x: view.width, y });
          if guide.padding[0] != 0 {
            renderer::add_line(out, view, from, to, hover_width, 38);
          }
          renderer::add_line(out, view, from, to, 1.0 / scale, 36);
        }
      }
    }

    for gap in &self.data.guide_gaps {
      if gap.display_id != display_id || gap.flags & 2 != 0 {
        continue;
      }
      let probe = guide_gap_probe(*gap);
      let (start, end, position) = self.project_probe(probe, offset);
      let (start, end) = if start > end {
        (end, start)
      } else {
        (start, end)
      };
      let start = renderer::snap(start, scale);
      let end = renderer::snap(end, scale);
      let position = renderer::snap(position, scale);
      let along = |value: f64| axis_point(gap.axis, value, position);
      if gap.padding[0] != 0 {
        renderer::add_line(out, view, along(start), along(end), hover_width, 38);
      }
      renderer::add_line(out, view, along(start), along(end), 1.0 / scale, 36);
      for edge in [start, end] {
        renderer::add_line(
          out,
          view,
          axis_point(gap.axis, edge, position - tick),
          axis_point(gap.axis, edge, position + tick),
          1.0 / scale,
          36,
        );
      }
    }

    for radius in &self.data.radii {
      if radius.display_id != display_id || radius.radius <= 0.0 {
        continue;
      }
      let world = radius_center(*radius);
      let center = Point {
        x: renderer::snap((world.x - offset.x - self.viewport_origin.x) * zoom, scale),
        y: renderer::snap((world.y - offset.y - self.viewport_origin.y) * zoom, scale),
      };
      let value = ((radius.radius * zoom * scale).round() / scale).max(1.0 / scale);
      renderer::add_ruler_arc(
        out,
        view,
        center,
        value,
        radius.corner,
        scale,
        radius.padding[0] != 0,
        hover_width,
        radius.flags & 1 != 0,
      );
    }

    for line in &self.data.centerlines {
      let frame = self.project_world_rect(line.x, line.y, line.width, line.height, offset);
      let center_x = renderer::snap(frame.origin.x + frame.size.width * 0.5, scale);
      let center_y = renderer::snap(frame.origin.y + frame.size.height * 0.5, scale);
      renderer::add_line(
        out,
        view,
        Point {
          x: center_x,
          y: frame.origin.y,
        },
        Point {
          x: center_x,
          y: frame.bottom(),
        },
        1.0 / scale,
        if line.flags & 1 != 0 { 43 } else { 42 },
      );
      renderer::add_line(
        out,
        view,
        Point {
          x: frame.origin.x,
          y: center_y,
        },
        Point {
          x: frame.right(),
          y: center_y,
        },
        1.0 / scale,
        if line.flags & 2 != 0 { 43 } else { 42 },
      );
    }

    for object in &self.data.inner_objects {
      let frame = self.project_world_rect(object.x, object.y, object.width, object.height, offset);
      add_center_object_outline(out, view, frame, scale);
      let center_x = renderer::snap(frame.origin.x + frame.size.width * 0.5, scale);
      let center_y = renderer::snap(frame.origin.y + frame.size.height * 0.5, scale);
      if object.flags & 1 != 0 {
        let half = frame.size.height.min(12.0 * zoom) * 0.5;
        renderer::add_line(
          out,
          view,
          Point {
            x: center_x,
            y: center_y - half,
          },
          Point {
            x: center_x,
            y: center_y + half,
          },
          2.5 / scale,
          43,
        );
      }
      if object.flags & 2 != 0 {
        let half = frame.size.width.min(12.0 * zoom) * 0.5;
        renderer::add_line(
          out,
          view,
          Point {
            x: center_x - half,
            y: center_y,
          },
          Point {
            x: center_x + half,
            y: center_y,
          },
          2.5 / scale,
          43,
        );
      }
    }

    for measurement in &self.data.measurements {
      let frame = self.project_world_rect(
        measurement.x,
        measurement.y,
        measurement.width,
        measurement.height,
        offset,
      );
      renderer::add_ruler_box(
        out,
        view,
        frame,
        scale,
        measurement.padding[0] != 0,
        hover_width,
      );
    }
  }

  /// The floating chrome: the four pooled label sets and then the loupe, which
  /// macOS kept above them in the view order.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn add_chrome_vertices(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    display_id: u32,
    offset: Point,
    scale: f64,
    light_mode: bool,
    now: Instant,
  ) {
    self.label_rects.clear();
    if !self.visible {
      return;
    }
    let value = metrics();
    let Some(atlas) = self.text.hex_atlas(
      device,
      scale,
      light_mode,
      value.font_size,
      value.line_height,
    ) else {
      return;
    };
    let Some(cell) = atlas.atlas.map(|metrics| metrics.glyph_width) else {
      return;
    };
    let glyph_rect = |index: usize| {
      atlas
        .atlas
        .map_or_else(Rect::default, |metrics| metrics.glyph_texture_rect(index))
    };
    let appearance = if light_mode {
      Appearance::Light
    } else {
      Appearance::Dark
    };
    // macOS read these from a one-control `ControlGroup` that nothing ever
    // hovered, so the resolved visual is the normal one.
    let visual = control_visual(
      ControlStyle::button(ControlColor::Neutral, ControlSize::Compact),
      Interaction::Normal,
      appearance,
    );
    let fills = [visual.fill, visual.foreground];
    let (control, inset) = spacing();
    let width_digits = decimal_digit_count(self.desktop_size.width);
    let height_digits = decimal_digit_count(self.desktop_size.height);

    for item in self.labels.clone() {
      let (id, kind, text, frame) = match item {
        LabelItem::Measurement(measurement) => {
          let text = measurement_text(
            Rect::from_xywh(
              measurement.x,
              measurement.y,
              measurement.width,
              measurement.height,
            ),
            measurement.flags & 1 != 0,
            width_digits,
            height_digits,
          );
          let plate = measurement_label_rect(
            self,
            measurement,
            &text,
            cell,
            value,
            control,
            inset,
            view,
            offset,
          );
          (measurement.id, 1_u8, text, plate)
        }
        LabelItem::Probe(probe) | LabelItem::GuideGap(probe) => {
          let text = stamped_probe_text(probe);
          let plate = probe_label_rect(
            self, probe, None, &text, cell, value, control, inset, view, offset,
          );
          (
            probe.id,
            if matches!(item, LabelItem::Probe(_)) {
              2
            } else {
              3
            },
            text,
            plate,
          )
        }
        LabelItem::Radius(radius) => {
          let text = radius_text(radius);
          let probe = radius_label_probe(radius);
          let plate = probe_label_rect(
            self,
            probe,
            Some(radius),
            &text,
            cell,
            value,
            control,
            inset,
            view,
            offset,
          );
          (radius.id, 4_u8, text, plate)
        }
      };
      self.label_rects.push(LabelRect {
        id,
        kind,
        rect: frame,
      });
      let start = out.len();
      renderer::add_plate(out, view, frame);
      let text_top = frame.origin.y + (frame.size.height - atlas.size.height) * 0.5;
      for (index, glyph) in text.chars().enumerate() {
        renderer::add_texture_quad(
          out,
          view,
          Rect::from_xywh(
            frame.origin.x + value.padding_x + cell * index as f64,
            text_top,
            cell,
            atlas.size.height,
          ),
          glyph_rect(super::text::glyph_index(glyph).unwrap_or(0)),
          11,
        );
      }
      push_segment(
        segments,
        out,
        start,
        fills,
        value.radius,
        Some(atlas.view.clone()),
        None,
      );
    }

    self.add_loupe(
      device, out, segments, view, display_id, scale, light_mode, now, &atlas, cell, fills, value,
      inset,
    );
  }

  /// The cursor readout (`render`, `+ruler.m:654-828`): a rounded plate, the
  /// picked-pixel swatch, its `#RRGGBB` hex, an optional `W × H px` row from
  /// the two live probes, the animated checkmark and the tolerance notice.
  #[allow(clippy::too_many_arguments)]
  fn add_loupe(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    display_id: u32,
    scale: f64,
    light_mode: bool,
    now: Instant,
    atlas: &super::text::TextTexture,
    cell: f64,
    fills: [[f32; 4]; 2],
    value: ControlMetrics,
    inset: f64,
  ) {
    if !self.transient_chrome
      || self.interaction_active
      || self.hovered_artifact_key != 0
      || !point_in_surface(self.point, view)
    {
      return;
    }
    let tolerance = self.tolerance_amount(now);
    let tolerance_label = (self.tolerance_visible || tolerance > 0.001)
      .then(|| {
        self.text.ink_label(
          device,
          tolerance_text(self.tolerance_mode),
          scale,
          light_mode,
          value.font_size,
          value.line_height,
        )
      })
      .flatten();

    let colour = hex_text(self.color);
    let dimensions = probe_dimensions_text(&self.data.probes, display_id, self.desktop_size);
    let colour_width = value.icon_size + value.gap + cell * 7.0;
    let reserved = reserved_dimensions_length(self.desktop_size);
    let dimensions_width = cell
      * dimensions
        .as_ref()
        .map_or(0, |text| text.chars().count())
        .max(if dimensions.is_some() { reserved } else { 0 }) as f64;
    let width = value.padding_x * 2.0 + colour_width.max(dimensions_width);
    let height = if dimensions.is_some() {
      value.height + value.line_height
    } else {
      value.height
    };
    let origin = loupe_origin(self.point, width, height, view, inset);
    let plate = Rect::from_xywh(origin.x, origin.y, width, height);

    let start = out.len();
    renderer::add_plate(out, view, plate);
    let colour_top = if dimensions.is_some() {
      value.line_height
    } else {
      0.0
    };
    let icon_top = colour_top + (value.height - value.icon_size) * 0.5;
    let swatch = Rect::from_xywh(
      origin.x + value.padding_x,
      origin.y + icon_top,
      value.icon_size,
      value.icon_size,
    );
    renderer::add_quad(out, view, swatch, 29);

    let glyph_rect = |index: usize| {
      atlas
        .atlas
        .map_or_else(Rect::default, |metrics| metrics.glyph_texture_rect(index))
    };
    let text_left = origin.x + value.padding_x + value.icon_size + value.gap;
    let text_top = origin.y + colour_top + (value.height - atlas.size.height) * 0.5;
    for (index, glyph) in colour.chars().enumerate() {
      renderer::add_texture_quad(
        out,
        view,
        Rect::from_xywh(
          text_left + cell * index as f64,
          text_top,
          cell,
          atlas.size.height,
        ),
        glyph_rect(super::text::glyph_index(glyph).unwrap_or(0)),
        48,
      );
    }
    if let Some(dimensions) = dimensions.as_ref() {
      let count = dimensions.chars().count();
      let left = origin.x + (width - cell * count as f64) * 0.5;
      let top = origin.y
        + (value.height - value.line_height) * 0.5
        + (value.line_height - atlas.size.height) * 0.5;
      for (index, glyph) in dimensions.chars().enumerate() {
        renderer::add_texture_quad(
          out,
          view,
          Rect::from_xywh(left + cell * index as f64, top, cell, atlas.size.height),
          glyph_rect(super::text::glyph_index(glyph).unwrap_or(0)),
          48,
        );
      }
    }

    // Matches CheckOnClick: scale and fade in, then only fade on expiry.
    let copied = self.copied.amount(now);
    let check_scale = if self.copied.target { copied } else { 1.0 };
    let center_x = swatch.origin.x + swatch.size.width * 0.5;
    let center_y = swatch.origin.y + swatch.size.height * 0.5;
    let a = Point {
      x: center_x - 4.0 * check_scale,
      y: center_y - 0.5 * check_scale,
    };
    let b = Point {
      x: center_x - 1.0 * check_scale,
      y: center_y + 2.5 * check_scale,
    };
    let c = Point {
      x: center_x + 5.0 * check_scale,
      y: center_y - 3.5 * check_scale,
    };
    renderer::add_line(out, view, a, b, 2.0 * check_scale.max(0.001), 30);
    renderer::add_line(out, view, b, c, 2.0 * check_scale.max(0.001), 30);

    let secondary = tolerance_label.as_ref().and_then(|label| {
      (tolerance > 0.001).then(|| {
        let label_scale = if self.tolerance.target {
          tolerance
        } else {
          1.0
        };
        let size = Size {
          width: label.size.width * label_scale,
          height: label.size.height * label_scale,
        };
        renderer::add_texture_quad(
          out,
          view,
          Rect::from_xywh(
            origin.x + (width - size.width) * 0.5,
            origin.y + (height - size.height) * 0.5,
            size.width,
            size.height,
          ),
          Rect::from_xywh(0.0, 0.0, 1.0, 1.0),
          37,
        );
        label.view.clone()
      })
    });
    push_segment(
      segments,
      out,
      start,
      fills,
      value.radius,
      Some(atlas.view.clone()),
      secondary,
    );
  }

  fn project_probe(&self, probe: ProbePacket, offset: Point) -> (f64, f64, f64) {
    let zoom = self.zoom();
    if probe.axis == 1 {
      (
        (probe.start - offset.x - self.viewport_origin.x) * zoom,
        (probe.end - offset.x - self.viewport_origin.x) * zoom,
        (probe.position - offset.y - self.viewport_origin.y) * zoom,
      )
    } else {
      (
        (probe.start - offset.y - self.viewport_origin.y) * zoom,
        (probe.end - offset.y - self.viewport_origin.y) * zoom,
        (probe.position - offset.x - self.viewport_origin.x) * zoom,
      )
    }
  }

  fn project_world_rect(&self, x: f64, y: f64, width: f64, height: f64, offset: Point) -> Rect {
    project_world_rect(
      x,
      y,
      width,
      height,
      offset,
      self.viewport_origin,
      self.zoom(),
    )
  }

  fn project_point(&self, point: Point, offset: Point) -> Point {
    let zoom = self.zoom();
    Point {
      x: (point.x - offset.x - self.viewport_origin.x) * zoom,
      y: (point.y - offset.y - self.viewport_origin.y) * zoom,
    }
  }
}

mod assignment;
mod commands;
mod data;
mod label_layout;
mod labels;
mod world;

pub(crate) use assignment::assign_labels;
pub(crate) use commands::key_command;
#[cfg(test)]
use commands::KeyCommand;
pub(crate) use data::DataChange;
pub(crate) use label_layout::{label_hit, loupe_origin};
pub(crate) use labels::{
  decimal_digit_count, hex_text, measurement_text, probe_dimensions_text, radius_text,
  reserved_dimensions_length, stamped_probe_text, tolerance_text,
};
pub(crate) use world::{
  animation_active, guide_gap_probe, hovered_artifact_key, project_world_rect, radius_center,
  radius_label_probe,
};

use assignment::point_in_surface;
use data::{labelled_guide_gaps, labelled_measurements, labelled_probes, labelled_radii, same};
use label_layout::{measurement_label_rect, probe_label_rect, push_segment};
use world::{add_center_object_outline, axis_point};

#[cfg(test)]
#[path = "ruler/tests.rs"]
mod tests;
