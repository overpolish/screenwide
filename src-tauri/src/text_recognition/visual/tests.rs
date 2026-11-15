// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
    tables: Vec::new(),
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
fn a_table_is_washed_with_room_round_its_cells_inside_the_area() {
  let cell = |x: f64, y: f64| screenwide_tables::Cell {
    text: String::new(),
    column: 0,
    span: 1,
    sources: Vec::new(),
    bounds: screenwide_tables::Rect {
      x,
      y,
      width: 0.2,
      height: 0.1,
    },
  };
  let result = TextRecognitionResult {
    lines: Vec::new(),
    qr_codes: Vec::new(),
    text: String::new(),
    tables: vec![screenwide_tables::Table {
      columns: 2,
      rows: vec![vec![cell(0.0, 0.2), cell(0.5, 0.2)], vec![cell(0.0, 0.4)]],
    }],
  };
  let visual = snapshot(Rect::from_xywh(100.0, 0.0, 400.0, 200.0), &result, &[]);
  // Half a cell's height, 10 points, round the cells, cut at the area's
  // start.
  assert_eq!(
    visual.rects,
    [VisualRect {
      rect: Rect::from_xywh(100.0, 30.0, 290.0, 80.0),
      kind: VisualKind::Table,
    }]
  );
}

#[test]
fn ready_and_choosing_packets_share_geometry_and_presentation_policy() {
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

  let choosing = RenderPacket::choosing();
  assert_eq!(choosing.phase, VisualPhase::Idle);
  assert!(choosing.rects.is_empty());
  assert_eq!(choosing.presentation.frame, Some(true));
  assert_eq!(choosing.presentation.input, Some(true));
  assert!(choosing.presentation.reset);
  assert!(choosing.presentation.claim_crosshair);
}
