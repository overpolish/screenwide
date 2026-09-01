// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::glide::region_rect::PlacedRegion;

/// The detector and native placement share one grid-region contract.
pub type GlideRegion = PlacedRegion;

pub const GRID_ROWS: u32 = 2;
pub fn bottom_row_region(grid_cols: u32) -> GlideRegion {
  GlideRegion {
    col_span: grid_cols,
    col_start: 0,
    grid_cols,
    row_span: 1,
    row_start: 1,
  }
}
pub fn regrid_region(region: GlideRegion, thirds: bool) -> GlideRegion {
  if thirds {
    GlideRegion {
      col_span: if region.col_span == 2 { 3 } else { 2 },
      grid_cols: 3,
      ..region
    }
  } else if region.col_span == 3 || (region.col_start == 1 && region.col_span == 1) {
    GlideRegion {
      col_span: 2,
      col_start: 0,
      grid_cols: 2,
      ..region
    }
  } else {
    GlideRegion {
      col_span: 1,
      col_start: if region.col_start == 0 { 0 } else { 1 },
      grid_cols: 2,
      ..region
    }
  }
}
const LADDER: [(u32, u32); 5] = [(1, 0), (2, 0), (1, 1), (2, 1), (1, 2)];
pub fn step_columns(region: GlideRegion, step: i32) -> GlideRegion {
  let far = if step > 0 { region.grid_cols - 1 } else { 0 };
  if region.grid_cols == 2 {
    if region.col_span == 2 {
      return GlideRegion {
        col_span: 1,
        col_start: far,
        ..region
      };
    }
    if region.col_start == far {
      return region;
    }
    if region.row_span < GRID_ROWS {
      return GlideRegion {
        col_span: 2,
        col_start: 0,
        ..region
      };
    }
    return GlideRegion {
      col_start: far,
      ..region
    };
  }
  let rung = LADDER
    .iter()
    .position(|&(span, start)| span == region.col_span && start == region.col_start);
  let next = rung
    .map(|n| (n as i32 + step).clamp(0, 4) as usize)
    .unwrap_or(if step > 0 { 3 } else { 1 });
  GlideRegion {
    col_span: LADDER[next].0,
    col_start: LADDER[next].1,
    ..region
  }
}
pub fn step_rows(region: GlideRegion, step: i32) -> GlideRegion {
  let far = if step > 0 { 1 } else { 0 };
  if region.row_span == GRID_ROWS {
    return GlideRegion {
      row_span: 1,
      row_start: far,
      ..region
    };
  }
  if region.row_start == far {
    region
  } else {
    GlideRegion {
      row_span: GRID_ROWS,
      row_start: 0,
      ..region
    }
  }
}
