// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::{
  regions::{self, GlideRegion},
  travel::axis_step,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GlideAction {
  Minimize,
}

/// Gesture thresholds and timing shared by every platform adapter.
#[derive(Clone, Copy, Debug)]
pub struct GlideDetectorOptions {
  pub diagonal_corner_ratio: f64,
  pub horizontal_dominance: f64,
  pub horizontal_threshold: f64,
  pub vertical_fill_threshold: f64,
  pub vertical_release_threshold: f64,
  pub vertical_threshold: f64,
  pub motion_noise_floor: f64,
  pub rest_ms: f64,
  pub reversal_hysteresis: f64,
}
impl Default for GlideDetectorOptions {
  fn default() -> Self {
    Self {
      diagonal_corner_ratio: 0.5,
      horizontal_dominance: 1.15,
      horizontal_threshold: 44.,
      vertical_fill_threshold: 44.,
      vertical_release_threshold: 20.,
      vertical_threshold: 44.,
      motion_noise_floor: 2.,
      rest_ms: 60.,
      reversal_hysteresis: 10.,
    }
  }
}
pub(super) struct GlideFold {
  pub pending: Option<GlideAction>,
  pub porous: bool,
  pub region: Option<GlideRegion>,
}
pub(crate) fn fold_horizontal(
  across: f64,
  down: f64,
  options: GlideDetectorOptions,
  thirds: bool,
) -> Option<GlideRegion> {
  if across.abs()
    < options
      .horizontal_threshold
      .max(down.abs() * options.horizontal_dominance)
  {
    return None;
  }
  let cols = if thirds { 3 } else { 2 };
  Some(GlideRegion {
    col_span: 1,
    col_start: if across > 0.0 { cols - 1 } else { 0 },
    grid_cols: cols,
    row_span: 2,
    row_start: 0,
  })
}
fn fold_corner(
  across: f64,
  down: f64,
  options: GlideDetectorOptions,
  thirds: bool,
) -> Option<GlideRegion> {
  let (width, height) = (across.abs(), down.abs());
  if width < options.horizontal_threshold
    || height < options.vertical_threshold
    || width.min(height) < width.max(height) * options.diagonal_corner_ratio
  {
    return None;
  }
  let cols = if thirds { 3 } else { 2 };
  Some(GlideRegion {
    col_span: 1,
    col_start: if across > 0.0 { cols - 1 } else { 0 },
    grid_cols: cols,
    row_span: 1,
    row_start: if down > 0.0 { 1 } else { 0 },
  })
}
fn fold_vertical(
  across: f64,
  down: f64,
  options: GlideDetectorOptions,
  thirds: bool,
) -> Option<GlideFold> {
  if down.abs()
    < options
      .vertical_fill_threshold
      .max(across.abs() * options.horizontal_dominance)
  {
    return None;
  }
  if down > 0.0 {
    return Some(GlideFold {
      pending: Some(GlideAction::Minimize),
      porous: false,
      region: None,
    });
  }
  let cols = if thirds { 3 } else { 2 };
  let middle = if thirds { 1 } else { 0 };
  Some(GlideFold {
    pending: None,
    porous: false,
    region: Some(GlideRegion {
      col_span: if middle == 1 { 1 } else { 2 },
      col_start: middle,
      grid_cols: cols,
      row_span: 2,
      row_start: 0,
    }),
  })
}
pub(super) fn detect_first_fold(
  across: f64,
  down: f64,
  options: GlideDetectorOptions,
  thirds: bool,
) -> Option<GlideFold> {
  if let Some(region) = fold_corner(across, down, options, thirds) {
    return Some(GlideFold {
      pending: None,
      porous: false,
      region: Some(region),
    });
  }
  if let Some(region) = fold_horizontal(across, down, options, thirds) {
    return Some(GlideFold {
      pending: None,
      porous: true,
      region: Some(region),
    });
  }
  fold_vertical(across, down, options, thirds)
}
pub(super) fn step_ladder(
  region: GlideRegion,
  across: f64,
  down: f64,
  options: GlideDetectorOptions,
) -> Option<GlideFold> {
  let side = axis_step(across, options.horizontal_threshold);
  if side != 0 {
    return Some(GlideFold {
      pending: None,
      porous: false,
      region: Some(regions::step_columns(region, side)),
    });
  }
  let full_height = region.row_span == regions::GRID_ROWS;
  let step = axis_step(
    down,
    if full_height {
      options.vertical_threshold
    } else {
      options.vertical_release_threshold
    },
  );
  if step == 0 {
    return None;
  }
  let next = regions::step_rows(region, step);
  Some(GlideFold {
    pending: if step > 0 && next == region {
      Some(GlideAction::Minimize)
    } else {
      None
    },
    porous: false,
    region: Some(next),
  })
}
