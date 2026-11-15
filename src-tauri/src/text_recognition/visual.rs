// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Portable OCR draw data. macOS and Windows adapters consume these desktop-
//! space rectangles without owning recognition or projection policy.

use screenwide_osc::geometry::Rect;

use super::{TextRecognitionResult, TextRect};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum VisualPhase {
  #[default]
  Idle = 0,
  Loading = 1,
  Ready = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum VisualKind {
  Line = 1,
  Qr = 2,
  QrError = 3,
  Selection = 4,
  Table = 5,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualRect {
  pub rect: Rect,
  pub kind: VisualKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualSnapshot {
  pub selection: Rect,
  pub rects: Vec<VisualRect>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct OcrRectPacket {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub kind: u8,
  pub padding: [u8; 7],
}

impl From<&VisualRect> for OcrRectPacket {
  fn from(value: &VisualRect) -> Self {
    Self {
      x: value.rect.origin.x,
      y: value.rect.origin.y,
      width: value.rect.size.width,
      height: value.rect.size.height,
      kind: value.kind as u8,
      padding: [0; 7],
    }
  }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SurfacePresentation {
  pub frame: Option<bool>,
  pub input: Option<bool>,
  pub reset: bool,
  pub claim_crosshair: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RenderPacket {
  pub phase: VisualPhase,
  pub rects: Vec<OcrRectPacket>,
  pub presentation: SurfacePresentation,
}

impl RenderPacket {
  pub(crate) fn loading() -> Self {
    Self {
      phase: VisualPhase::Loading,
      presentation: SurfacePresentation {
        frame: Some(false),
        input: Some(false),
        ..Default::default()
      },
      ..Default::default()
    }
  }

  pub(crate) fn ready(snapshot: &VisualSnapshot) -> Self {
    Self {
      phase: VisualPhase::Ready,
      rects: snapshot.rects.iter().map(Into::into).collect(),
      presentation: SurfacePresentation {
        frame: Some(false),
        input: Some(true),
        ..Default::default()
      },
    }
  }

  /// Back to drawing an area over the same frozen desktop, after reading one
  /// failed or to read another: the selection and what was found are dropped.
  pub(crate) fn choosing() -> Self {
    Self {
      phase: VisualPhase::Idle,
      presentation: SurfacePresentation {
        frame: Some(true),
        input: Some(true),
        reset: true,
        claim_crosshair: true,
      },
      ..Default::default()
    }
  }
}

pub fn snapshot(
  selection: Rect,
  result: &TextRecognitionResult,
  selected: &[TextRect],
) -> VisualSnapshot {
  let mut rects = Vec::with_capacity(
    result.tables.len() + result.lines.len() + result.qr_codes.len() + selected.len(),
  );
  // Under the lines, so a table's text keeps its own wash over the table's.
  rects.extend(
    result
      .tables
      .iter()
      .filter_map(|table| table_rect(selection, table))
      .map(|rect| VisualRect {
        rect,
        kind: VisualKind::Table,
      }),
  );
  rects.extend(result.lines.iter().filter_map(|line| {
    project(selection, line.bounds).map(|rect| VisualRect {
      rect,
      kind: VisualKind::Line,
    })
  }));
  rects.extend(result.qr_codes.iter().filter_map(|code| {
    project(selection, code.bounds).map(|rect| VisualRect {
      rect,
      kind: if code.decode_error.is_some() {
        VisualKind::QrError
      } else {
        VisualKind::Qr
      },
    })
  }));
  rects.extend(selected.iter().filter_map(|bounds| {
    project(selection, *bounds).map(|rect| VisualRect {
      rect,
      kind: VisualKind::Selection,
    })
  }));
  VisualSnapshot { selection, rects }
}

/// The area a table takes, in desktop points: its cells' text with half a
/// cell's height of room round it, as a table pads its cells, kept inside
/// the area read.
fn table_rect(selection: Rect, table: &screenwide_tables::Table) -> Option<Rect> {
  let bounds = table.bounds()?;
  let mut heights: Vec<f64> = table
    .rows
    .iter()
    .flatten()
    .map(|cell| cell.bounds.height)
    .collect();
  heights.sort_by(f64::total_cmp);
  let pad = 0.5 * heights[heights.len() / 2] * selection.size.height;
  let rect = project(
    selection,
    TextRect {
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
    },
  )?;
  let left = (rect.origin.x - pad).max(selection.origin.x);
  let top = (rect.origin.y - pad).max(selection.origin.y);
  let right = (rect.right() + pad).min(selection.right());
  let bottom = (rect.bottom() + pad).min(selection.bottom());
  (right > left && bottom > top).then(|| Rect::from_xywh(left, top, right - left, bottom - top))
}

fn project(selection: Rect, normalized: TextRect) -> Option<Rect> {
  let rect = Rect::from_xywh(
    selection.origin.x + normalized.x * selection.size.width,
    selection.origin.y + normalized.y * selection.size.height,
    normalized.width * selection.size.width,
    normalized.height * selection.size.height,
  );
  rect.committed().then_some(rect)
}

#[cfg(test)]
mod tests;
