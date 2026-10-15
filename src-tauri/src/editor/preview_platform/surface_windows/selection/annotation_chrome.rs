// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotation chrome, drawn upright or through a turned layer's turn.
//! Upright, its lines are snapped quads on the pixel grid. Turned, each piece
//! is placed through the turn first and drawn in soft lines, since snapped
//! quads carried through a turn step along every side; a grip stays a round
//! disc where its middle lands rather than squashing with the picture.

use super::super::turn::{carried_box, carried_point, carried_rect, carried_segment};
use super::*;
use screenwide_scenes::rotation::Homography;

/// What the annotation chrome draws, in device pixels of the layer standing
/// upright.
pub(in crate::editor::preview_platform::surface_windows) struct AnnotationChrome<'a> {
  /// The chosen annotation's grips, possibly none with a tool in hand and
  /// nothing chosen, plus the disc over a snapped tip.
  pub(in crate::editor::preview_platform::surface_windows) handles: &'a [[f32; 2]],
  /// The chosen box annotation's box and its radius percentage, drawn as the
  /// layer selection draws its own; a stroke's has no radius dot.
  pub(in crate::editor::preview_platform::surface_windows) box_frame:
    Option<([f32; 4], Option<f64>)>,
  /// The chosen image's own frame: its four corners, clockwise from the
  /// picture's own top-left, then its radius dot.
  pub(in crate::editor::preview_platform::surface_windows) image_frame: Option<[[f32; 2]; 5]>,
  /// The element an arrow's tip has snapped to, outlined a device pixel wide
  /// so the anchor it took reads as part of that element.
  pub(in crate::editor::preview_platform::surface_windows) bounds: Option<[f32; 4]>,
  /// The equal-gap bars a snapped counter lined up with: a hairline across
  /// each gap with a tick at each end.
  pub(in crate::editor::preview_platform::surface_windows) gaps: &'a [[f32; 4]],
  /// The boxes round annotations chosen together, whole groups first.
  pub(in crate::editor::preview_platform::surface_windows) group: &'a [[f32; 4]],
  /// The marquee band being drawn.
  pub(in crate::editor::preview_platform::surface_windows) marquee: Option<[f32; 4]>,
}

fn logical_point([x, y]: [f32; 2], scale: f64) -> Point {
  Point {
    x: f64::from(x) / scale,
    y: f64::from(y) / scale,
  }
}

/// The element's outline as four rects a device pixel thick, inside it.
fn bounds_edges(rect: Rect, scale: f64) -> [Rect; 4] {
  let pixel = 1.0 / scale;
  let (width, height) = (rect.size.width, rect.size.height);
  let (x, y) = (rect.origin.x, rect.origin.y);
  [
    Rect::from_xywh(x, y, width, pixel),
    Rect::from_xywh(x, y + height - pixel, width, pixel),
    Rect::from_xywh(x, y, pixel, height),
    Rect::from_xywh(x + width - pixel, y, pixel, height),
  ]
}

/// The chrome on a layer standing upright.
pub(super) fn add_upright(
  vertices: &mut Vec<Vertex>,
  view: Size,
  chrome: &AnnotationChrome,
  scale: f64,
) {
  // The element first: it is the context for the anchor that landed on it,
  // so the disc sits over its outline rather than under it. The object
  // colour is the one the guides use too.
  if let Some(bounds) = chrome.bounds.filter(|rect| rect[2] > 0.0 && rect[3] > 0.0) {
    for edge in bounds_edges(logical_rect(bounds, scale), scale) {
      osc_gpu::add_pixel_aligned_quad(vertices, view, edge, scale, 5);
    }
  }
  for bar in chrome.gaps {
    osc_gpu::add_pixel_aligned_quad(vertices, view, logical_rect(*bar, scale), scale, 5);
  }
  if let Some((frame, radius_percent)) = chrome.box_frame {
    osc_gpu::add_selection(
      vertices,
      view,
      logical_rect(frame, scale),
      scale,
      radius_percent.unwrap_or(0.0),
      radius_percent.is_some(),
    );
  }
  if let Some(frame) = chrome.image_frame {
    let [a, b, c, d, dot] = frame.map(|point| logical_point(point, scale));
    osc_gpu::add_turned_selection(vertices, view, [a, b, c, d], Some(dot), scale);
  }
  for frame in chrome.group {
    osc_gpu::add_group_frame(vertices, view, logical_rect(*frame, scale), scale);
  }
  // The region selector's marching ants with no shade and no handles: the
  // band is its own image, so nothing round it is dimmed.
  if let Some(band) = chrome.marquee.filter(|rect| rect[2] > 0.0 || rect[3] > 0.0) {
    let band = logical_rect(band, scale);
    osc_gpu::add_crop_with_handles(vertices, view, band, band, scale, 0.0, true, false);
  }
  let points = chrome
    .handles
    .iter()
    .map(|point| logical_point(*point, scale))
    .collect::<Vec<_>>();
  osc_gpu::add_annotation_handles(vertices, view, &points, scale);
}

/// The chrome on a layer turned by `forward`, in the same order as upright.
pub(super) fn add_turned(
  vertices: &mut Vec<Vertex>,
  view: Size,
  chrome: &AnnotationChrome,
  forward: Homography,
  scale: f64,
) {
  if let Some(bounds) = chrome.bounds.filter(|rect| rect[2] > 0.0 && rect[3] > 0.0) {
    // The outline's middle line, a device pixel wide, as the upright one's
    // pixel rows are.
    let half = 0.5 / scale;
    let rect = logical_rect(bounds, scale);
    let inset = Rect::from_xywh(
      rect.origin.x + half,
      rect.origin.y + half,
      rect.size.width - half * 2.0,
      rect.size.height - half * 2.0,
    );
    let corners = carried_rect(forward, inset);
    for side in 0..4 {
      osc_gpu::add_soft_guide(
        vertices,
        view,
        corners[side],
        corners[(side + 1) % 4],
        1.0 / scale,
        true,
        scale,
      );
    }
  }
  for bar in chrome.gaps {
    // A bar or tick is a thin rect: its middle line, as wide as it is thin.
    let rect = logical_rect(*bar, scale);
    let (width, height) = (rect.size.width, rect.size.height);
    let (x, y) = (rect.origin.x, rect.origin.y);
    let (from, to, thickness) = if width >= height {
      let middle = y + height * 0.5;
      ((x, middle), (x + width, middle), height)
    } else {
      let middle = x + width * 0.5;
      ((middle, y), (middle, y + height), width)
    };
    let carry = |(x, y): (f64, f64)| carried_point(forward, Point { x, y });
    osc_gpu::add_soft_guide(
      vertices,
      view,
      carry(from),
      carry(to),
      thickness,
      true,
      scale,
    );
  }
  if let Some((frame, radius_percent)) = chrome.box_frame {
    let (corners, dot) = carried_box(forward, logical_rect(frame, scale), radius_percent);
    osc_gpu::add_turned_selection(vertices, view, corners, dot, scale);
  }
  if let Some(frame) = chrome.image_frame {
    let [a, b, c, d, dot] = frame.map(|point| carried_point(forward, logical_point(point, scale)));
    osc_gpu::add_turned_selection(vertices, view, [a, b, c, d], Some(dot), scale);
  }
  for frame in chrome.group {
    let corners = carried_rect(forward, logical_rect(*frame, scale));
    osc_gpu::add_turned_outline(vertices, view, corners, scale);
  }
  if let Some(band) = chrome.marquee.filter(|rect| rect[2] > 0.0 || rect[3] > 0.0) {
    let corners = carried_rect(forward, logical_rect(band, scale));
    osc_gpu::add_turned_marquee(vertices, view, corners, scale);
  }
  let points = chrome
    .handles
    .iter()
    .map(|point| carried_point(forward, logical_point(*point, scale)))
    .collect::<Vec<_>>();
  osc_gpu::add_annotation_handles(vertices, view, &points, scale);
}

/// An annotation gesture's guides on a layer turned by `forward`: each runs
/// the whole view across the layer standing upright, and is carried with it.
/// `guides` are as `SelectionOverlay::draw` takes them, in device pixels.
pub(super) fn add_turned_guides(
  vertices: &mut Vec<Vertex>,
  view: Size,
  (x, y, x_object, y_object): (Option<f32>, Option<f32>, bool, bool),
  forward: Homography,
  scale: f64,
) {
  let lines = [
    x.map(|x| {
      let x = f64::from(x) / scale;
      (Point { x, y: 0.0 }, Point { x, y: view.height }, x_object)
    }),
    y.map(|y| {
      let y = f64::from(y) / scale;
      (Point { x: 0.0, y }, Point { x: view.width, y }, y_object)
    }),
  ];
  for (from, to, object) in lines.into_iter().flatten() {
    if let Some((from, to)) = carried_segment(forward, from, to, view) {
      osc_gpu::add_soft_guide(vertices, view, from, to, 1.0 / scale, object, scale);
    }
  }
}
