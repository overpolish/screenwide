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

#[path = "ruler/chrome.rs"]
mod chrome;
#[path = "ruler/loupe.rs"]
mod loupe;
#[path = "ruler/world_vertices.rs"]
mod world_vertices;

#[path = "ruler/state.rs"]
mod state;

#[path = "ruler/animation.rs"]
mod animation;
use animation::{ease, Animation};

use std::time::{Duration, Instant};

use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11ShaderResourceView};

use super::ocr::Segment;
use super::renderer::{self, Vertex};
use super::text::{AtlasMetrics, TextCache};
use crate::osc::{
  controls::{
    control_metrics, control_stroke, control_visual, Appearance, ControlColor, ControlKind,
    ControlMetrics, ControlSize, ControlStyle, Interaction,
  },
  geometry::{Point, Rect, Size},
};
use crate::ruler::render::{
  CenterlinePacket, GuideGapPacket, GuidePacket, InnerObjectPacket, MeasurementPacket, ProbePacket,
  RadiusPacket, ViewportPacket,
};

/// Every ruler transition - the copied checkmark, the hover pulse and the
/// tolerance notice - runs over this window (`+ruler.m:7-9`).
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
/// tree - the same route the icon atlas takes in `surface.rs`.
#[repr(C)]
#[derive(Clone, Copy)]
struct NativeControlSpacing {
  tight: f64,
  control: f64,
  control_inset: f64,
  section: f64,
  layout: f64,
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
  control_metrics(ControlKind::Button, ControlSize::Regular)
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

/// An instant far enough in the past that every transition reads as settled.
fn settled() -> Instant {
  Instant::now()
    .checked_sub(ANIMATION_DURATION)
    .unwrap_or_else(Instant::now)
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
mod atlas_layout;
mod commands;
mod data;
mod label_layout;
mod labels;
use atlas_layout::add_atlas_text;
mod world;

pub(crate) use assignment::assign_labels;
pub(crate) use commands::key_command;
#[cfg(test)]
use commands::KeyCommand;
pub(crate) use data::DataChange;
pub(crate) use label_layout::{label_hit, loupe_origin};
pub(crate) use labels::{
  hex_text, measurement_text, probe_dimensions_text, radius_text, stamped_probe_text,
  tolerance_text,
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
