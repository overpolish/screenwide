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
  pub message: String,
  pub presentation: SurfacePresentation,
}

impl RenderPacket {
  pub(crate) fn loading(message: impl Into<String>) -> Self {
    Self {
      phase: VisualPhase::Loading,
      message: message.into(),
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
      ..Default::default()
    }
  }

  pub(crate) fn error(message: impl Into<String>) -> Self {
    Self {
      phase: VisualPhase::Error,
      message: message.into(),
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

  #[test]
  fn ready_and_error_packets_share_geometry_and_presentation_policy() {
    let snapshot = VisualSnapshot {
      selection: Rect::from_xywh(0.0, 0.0, 100.0, 100.0),
      rects: vec![VisualRect {
        rect: Rect::from_xywh(10.0, 20.0, 30.0, 40.0),
        kind: VisualKind::Selection,
      }],
    };

    let ready = RenderPacket::ready(&snapshot);
    assert_eq!(ready.phase, VisualPhase::Ready);
    assert_eq!((ready.rects[0].x, ready.rects[0].kind), (10.0, 4));
    assert_eq!(ready.presentation.input, Some(true));
    assert_eq!(ready.presentation.frame, Some(false));
    assert!(!ready.presentation.claim_crosshair);

    let error = RenderPacket::error("failed");
    assert_eq!(error.phase, VisualPhase::Error);
    assert_eq!(error.message, "failed");
    assert_eq!(error.presentation.frame, Some(true));
    assert_eq!(error.presentation.input, Some(true));
    assert!(error.presentation.reset);
    assert!(error.presentation.claim_crosshair);
  }
}
