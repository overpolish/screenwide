// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editing::PreviewSelection;
use screenwide_compositor::workspace_editor;

pub(super) fn selection_resize(
  start: PreviewSelection,
  edges: u32,
  delta: (f64, f64),
  pane: (f64, f64),
  zoom: f64,
  centered: bool,
) -> workspace_editor::SelectionResize {
  let minimum = (36.0 / (pane.0 * zoom * start.width).max(1.0))
    .max(36.0 / (pane.1 * zoom * start.height).max(1.0));
  workspace_editor::selection_resize(
    workspace_editor::NormalizedRect {
      x: start.x,
      y: start.y,
      width: start.width,
      height: start.height,
    },
    edges,
    delta,
    minimum,
    centered,
  )
}
