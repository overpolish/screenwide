// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Portable OCR draw data. macOS and Windows adapters consume these desktop-
//! space rectangles without owning recognition or projection policy.

use crate::osc::geometry::Rect;

use super::{TextRecognitionResult, TextRect};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum VisualPhase {
  #[default]
  Idle = 0,
  Loading = 1,
  Ready = 2,
  Error = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum VisualKind {
  Line = 1,
  Qr = 2,
  QrError = 3,
  Selection = 4,
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

pub fn snapshot(
  selection: Rect,
  result: &TextRecognitionResult,
  selected: &[TextRect],
) -> VisualSnapshot {
  let mut rects = Vec::with_capacity(result.lines.len() + result.qr_codes.len() + selected.len());
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
mod tests {
  use super::*;
  use crate::text_recognition::{RecognizedLine, RecognizedQrCode};

  #[test]
  fn projects_results_into_one_desktop_selection() {
    let result = TextRecognitionResult {
      lines: vec![RecognizedLine {
        text: "hello".to_owned(),
        confidence: 1.0,
        bounds: TextRect {
          x: 0.1,
          y: 0.2,
          width: 0.5,
          height: 0.1,
        },
        characters: Vec::new(),
      }],
      qr_codes: vec![RecognizedQrCode {
        bounds: TextRect {
          x: 0.8,
          y: 0.0,
          width: 0.2,
          height: 1.0,
        },
        content: String::new(),
        decode_error: Some("unsupported".to_owned()),
      }],
      text: "hello".to_owned(),
    };
    let visual = snapshot(
      Rect::from_xywh(1700.0, 50.0, 400.0, 200.0),
      &result,
      &[TextRect {
        x: 0.2,
        y: 0.3,
        width: 0.1,
        height: 0.2,
      }],
    );

    assert_eq!(
      visual.rects[0].rect,
      Rect::from_xywh(1740.0, 90.0, 200.0, 20.0)
    );
    assert_eq!(visual.rects[0].kind, VisualKind::Line);
    assert_eq!(
      visual.rects[1].rect,
      Rect::from_xywh(2020.0, 50.0, 80.0, 200.0)
    );
    assert_eq!(visual.rects[1].kind, VisualKind::QrError);
    assert_eq!(visual.rects[2].kind, VisualKind::Selection);
    assert_eq!(
      visual.rects[2].rect,
      Rect::from_xywh(1780.0, 110.0, 40.0, 40.0)
    );
  }
}
