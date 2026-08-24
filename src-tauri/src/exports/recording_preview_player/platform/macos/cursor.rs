// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::exports::cursor_effects::{CursorCompositor, CursorEffectSettings, GpuCursor};

#[derive(Clone, Copy)]
pub(super) struct GpuCursorPreview {
  pub canvas_height: u32,
  pub canvas_width: u32,
  pub cursor: GpuCursor,
}

pub(super) fn gpu_cursor_preview(
  cursor: Option<&CursorCompositor>,
  position_ms: u64,
  settings: CursorEffectSettings,
  output: (u32, u32),
) -> Option<GpuCursorPreview> {
  let cursor =
    cursor
      .filter(|_| settings.bake)?
      .gpu_cursor(position_ms, (output.0, output.1), settings)?;
  Some(GpuCursorPreview {
    canvas_height: output.1,
    canvas_width: output.0,
    cursor,
  })
}
