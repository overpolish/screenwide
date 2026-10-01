// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Ruler {
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
}
