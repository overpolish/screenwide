// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the island's parts are in its window, as the island reports them,
//! so clicks on them reach it and clicks anywhere else pass through.

use std::sync::{Mutex, PoisonError};

use serde::Deserialize;
use tauri::AppHandle;

use super::native;

/// How far round the island the pointer still counts as on it, in points.
/// The pointer leaving the island is then seen by the island itself, inside
/// its window, before clicks start passing through, so what hovering opened
/// closes as it should.
const MARGIN: f64 = 8.0;

/// A part of the island, in its window, in CSS pixels from the top left.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct IslandRect {
  pub height: f64,
  pub width: f64,
  pub x: f64,
  pub y: f64,
}

/// The island's parts as last reported, kept while it is hidden so it comes
/// back knowing where they are.
static RECTS: Mutex<Vec<IslandRect>> = Mutex::new(Vec::new());

/// The island's parts as last reported.
pub(in crate::app_windows) fn rects() -> Vec<IslandRect> {
  RECTS.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// Whether a point in the island's window, in points from its top left, is
/// on one of the island's parts.
pub(in crate::app_windows) fn is_on_island(rects: &[IslandRect], x: f64, y: f64) -> bool {
  rects.iter().any(|rect| {
    x >= rect.x - MARGIN
      && x <= rect.x + rect.width + MARGIN
      && y >= rect.y - MARGIN
      && y <= rect.y + rect.height + MARGIN
  })
}

/// Where the island's parts are, so clicks on them reach it and clicks
/// anywhere else in its window pass through to what is underneath.
#[tauri::command]
pub fn set_island_hit_area(app: AppHandle, rects: Vec<IslandRect>) {
  rects.clone_into(&mut RECTS.lock().unwrap_or_else(PoisonError::into_inner));
  // The island changing shape can bring its edge to the pointer, or take it
  // away, without the pointer moving.
  native::refresh_hit_area(&app);
}
