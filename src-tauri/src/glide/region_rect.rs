// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The fraction-to-rectangle math a committed Glide region resolves to. It is
//! pure and platform-free on purpose: each platform reads its own work area and
//! feeds the numbers in here, so halves tile identically everywhere.

/// The grid the detector commits against always has two rows; only the column
/// count changes with the thirds modifier.
const GRID_ROWS: u32 = 2;

/// A committed destination, as a span of grid cells. `grid_cols` is however many
/// columns the detector was showing when the fingers lifted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedRegion {
  pub col_start: u32,
  pub col_span: u32,
  pub grid_cols: u32,
  pub row_start: u32,
  pub row_span: u32,
}

/// The rectangle a region covers inside a work area, as `(origin, size)`.
///
/// Sizing is edge-derived rather than fraction-scaled: both the origin and the
/// far edge are floored from the same fractions, and the size is their
/// difference. A left half and a right half therefore share one floored edge
/// value and tile with no seam and no overlap, whichever way the work area's
/// width divides.
///
/// `gap` insets every edge: the ones on the work area's border by the whole
/// gap, the ones two regions share by half each, so adjacent windows sit
/// exactly one gap apart. A gap of zero leaves the rectangle untouched.
pub fn region_rect(
  work_origin: (f64, f64),
  work_size: (f64, f64),
  region: &PlacedRegion,
  gap: u32,
) -> ((f64, f64), (f64, f64)) {
  let (left, right) = edges(
    work_origin.0,
    work_size.0,
    region.col_start,
    region.col_span,
    region.grid_cols,
    gap,
  );
  let (top, bottom) = edges(
    work_origin.1,
    work_size.1,
    region.row_start,
    region.row_span,
    GRID_ROWS,
    gap,
  );
  ((left, top), (right - left, bottom - top))
}

/// Which edge of a region a window that cannot fill it should hug, per axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gravity {
  /// The near edge: left, or top.
  Start,
  /// The far edge: right, or bottom.
  End,
  Center,
}

/// The pull a region exerts on a window too small for it, on both axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionGravity {
  pub horizontal: Gravity,
  pub vertical: Gravity,
}

/// Where a size-constrained window belongs inside a region. A region touching
/// only one edge of the work area pulls towards that edge, which is the one the
/// gesture aimed at; one that spans the whole axis, or floats in the middle of
/// it, has no edge to prefer and centers.
pub fn region_gravity(region: &PlacedRegion) -> RegionGravity {
  RegionGravity {
    horizontal: axis_gravity(region.col_start, region.col_span, region.grid_cols),
    vertical: axis_gravity(region.row_start, region.row_span, GRID_ROWS),
  }
}

fn axis_gravity(start: u32, span: u32, count: u32) -> Gravity {
  let count = count.max(1);
  let (start, span) = clamped_span(start, span, count);
  match (start == 0, start + span == count) {
    (true, false) => Gravity::Start,
    (false, true) => Gravity::End,
    // Both edges is a full span and neither is a floating middle: nothing to
    // hug either way.
    _ => Gravity::Center,
  }
}

/// The floored near and far edges of a cell span along one axis, inset by the
/// window gap.
///
/// The two halves of a shared edge are floored and ceiled rather than both
/// halved, so every inset stays a whole pixel and the two sides still add up to
/// exactly one gap. A gap too wide for the region collapses it to a zero-width
/// sliver instead of turning it inside out.
fn edges(origin: f64, extent: f64, start: u32, span: u32, count: u32, gap: u32) -> (f64, f64) {
  let count = count.max(1);
  let (start, span) = clamped_span(start, span, count);
  let near_inset = if start == 0 { gap } else { gap - gap / 2 };
  let far_inset = if start + span == count { gap } else { gap / 2 };
  let near = edge(origin, extent, start, count) + f64::from(near_inset);
  let far = edge(origin, extent, start + span, count) - f64::from(far_inset);
  (near, far.max(near))
}

/// A cell span clamped into a track of `count` cells, so a malformed payload
/// lands somewhere on the monitor rather than off it, and an empty span still
/// covers one cell.
fn clamped_span(start: u32, span: u32, count: u32) -> (u32, u32) {
  let start = start.min(count - 1);
  (start, span.clamp(1, count - start))
}

fn edge(origin: f64, extent: f64, index: u32, count: u32) -> f64 {
  (origin + extent * f64::from(index) / f64::from(count)).floor()
}

#[cfg(test)]
#[path = "region_rect_tests.rs"]
mod tests;
