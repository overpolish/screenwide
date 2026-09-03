// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// `uv` carries local distance in fitted pattern pixels; `aux` carries the
/// edge's cumulative phase and length. This lets the shader shorten only a
/// dash that crosses a corner and give it a genuine rounded terminal cap.
#[derive(Clone, Copy)]
struct PatternEdge {
  kind: u32,
  horizontal: bool,
  forward: bool,
  phase: f64,
  length: f64,
}

fn add_pattern_quad(out: &mut Vec<Vertex>, view: Size, rect: Rect, edge: PatternEdge) {
  let (start, end) = if edge.forward {
    (0.0, edge.length as f32)
  } else {
    (edge.length as f32, 0.0)
  };
  let uvs = if edge.horizontal {
    [[start, 0.0], [end, 0.0], [end, 1.0], [start, 1.0]]
  } else {
    [[0.0, start], [1.0, start], [1.0, end], [0.0, end]]
  };
  push_quad_with_aux(
    out,
    rect_corners(view, rect),
    uvs,
    [edge.phase as f32, edge.length as f32],
    edge.kind,
  );
}

/// The margin gives the SDF room for its anti-aliased rim.
fn add_circle(
  out: &mut Vec<Vertex>,
  view: Size,
  center: Point,
  radius: f64,
  margin: f64,
  kind: u32,
) {
  let extent = radius + margin;
  add_quad(
    out,
    view,
    Rect::from_xywh(
      center.x - extent,
      center.y - extent,
      extent * 2.0,
      extent * 2.0,
    ),
    kind,
  );
}

fn add_pill(out: &mut Vec<Vertex>, view: Size, center: Point, horizontal: bool, scale: f64) {
  let width = (if horizontal { 12.0 } else { 6.0 }) + 4.0 / scale;
  let height = (if horizontal { 6.0 } else { 12.0 }) + 4.0 / scale;
  add_quad(
    out,
    view,
    Rect::from_xywh(
      center.x - width * 0.5,
      center.y - height * 0.5,
      width,
      height,
    ),
    16,
  );
}

fn add_selection_frame(
  out: &mut Vec<Vertex>,
  view: Size,
  frame: Rect,
  scale: f64,
  halo_kind: u32,
  line_kind: u32,
  halo_width: f64,
) {
  let min_x = snap(frame.origin.x, scale);
  let max_x = snap(frame.right(), scale);
  let min_y = snap(frame.origin.y, scale);
  let max_y = snap(frame.bottom(), scale);
  for pass in 0..2 {
    let halo = pass == 0;
    let half = if halo { halo_width * 0.5 } else { 0.5 / scale };
    let rect_kind = if halo { halo_kind } else { line_kind };
    add_quad(
      out,
      view,
      Rect::from_xywh(
        min_x - half,
        min_y - half,
        max_x - min_x + half * 2.0,
        half * 2.0,
      ),
      rect_kind,
    );
    add_quad(
      out,
      view,
      Rect::from_xywh(
        min_x - half,
        max_y - half,
        max_x - min_x + half * 2.0,
        half * 2.0,
      ),
      rect_kind,
    );
    add_quad(
      out,
      view,
      Rect::from_xywh(
        min_x - half,
        min_y - half,
        half * 2.0,
        max_y - min_y + half * 2.0,
      ),
      rect_kind,
    );
    add_quad(
      out,
      view,
      Rect::from_xywh(
        max_x - half,
        min_y - half,
        half * 2.0,
        max_y - min_y + half * 2.0,
      ),
      rect_kind,
    );
  }
}

fn handle_points(
  min_x: f64,
  min_y: f64,
  max_x: f64,
  max_y: f64,
  mid_x: f64,
  mid_y: f64,
) -> [Point; 8] {
  [
    Point { x: min_x, y: min_y },
    Point { x: mid_x, y: min_y },
    Point { x: max_x, y: min_y },
    Point { x: max_x, y: mid_y },
    Point { x: max_x, y: max_y },
    Point { x: mid_x, y: max_y },
    Point { x: min_x, y: max_y },
    Point { x: min_x, y: mid_y },
  ]
}

fn add_handles(out: &mut Vec<Vertex>, view: Size, points: [Point; 8], scale: f64) {
  let radius = 4.0 + 1.0 / scale;
  for (index, point) in points.iter().enumerate() {
    let point = snap_handle_point(*point, scale);
    if index & 1 == 0 {
      add_circle(out, view, point, radius, 1.0 / scale, 3);
    } else {
      add_pill(out, view, point, index == 1 || index == 5, scale);
    }
  }
}

pub(crate) fn add_selection(
  out: &mut Vec<Vertex>,
  view: Size,
  frame: Rect,
  scale: f64,
  radius_percent: f64,
  radius_enabled: bool,
) {
  let min_x = snap(frame.origin.x, scale);
  let max_x = snap(frame.right(), scale);
  let min_y = snap(frame.origin.y, scale);
  let max_y = snap(frame.bottom(), scale);
  let mid_x = snap((min_x + max_x) / 2.0, scale);
  let mid_y = snap((min_y + max_y) / 2.0, scale);
  add_selection_frame(out, view, frame, scale, 2, 0, 3.0 / scale);
  add_handles(
    out,
    view,
    handle_points(min_x, min_y, max_x, max_y, mid_x, mid_y),
    scale,
  );
  if radius_enabled {
    let offset = (max_x - min_x).min(max_y - min_y) * radius_percent / 100.0 * 0.55 + 10.0;
    add_circle(
      out,
      view,
      snap_handle_point(
        Point {
          x: min_x + offset,
          y: min_y + offset,
        },
        scale,
      ),
      4.0 + 1.0 / scale,
      1.0 / scale,
      3,
    );
  }
}

mod crop;

pub(crate) use crop::{add_crop, add_crop_with_handles};
