// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::{
  osc::geometry::Rect,
  text_recognition::{RecognizedLine, RecognizedQrCode, TextRect},
};

fn state_with_qr() -> TextRecognitionState {
  let state = TextRecognitionState::default();
  {
    let mut session = state.0.lock().unwrap();
    session.generation = 7;
    session.selection = Some(Rect::from_xywh(1700.0, 50.0, 400.0, 200.0));
  }
  assert!(state.install_result(
    7,
    TextRecognitionResult {
      lines: Vec::new(),
      qr_codes: vec![RecognizedQrCode {
        bounds: TextRect {
          x: 0.5,
          y: 0.2,
          width: 0.2,
          height: 0.4,
        },
        content: "https://screenwide.app".to_owned(),
        decode_error: None,
      }],
      text: String::new(),
    }
  ));
  state
}

#[test]
fn desktop_pointer_input_updates_native_selection_and_copy_text() {
  let state = TextRecognitionState::default();
  {
    let mut session = state.0.lock().unwrap();
    session.generation = 7;
    session.selection = Some(Rect::from_xywh(1700.0, 50.0, 400.0, 200.0));
  }
  assert!(state.install_result(
    7,
    TextRecognitionResult {
      lines: vec![RecognizedLine {
        text: "hello".to_owned(),
        confidence: 1.0,
        bounds: TextRect {
          x: 0.1,
          y: 0.1,
          width: 0.5,
          height: 0.1,
        },
        characters: Vec::new(),
      }],
      qr_codes: Vec::new(),
      text: "hello".to_owned(),
    }
  ));

  state
    .text_input(
      TextAction::Down {
        additive: false,
        double: false,
      },
      Point { x: 1740.0, y: 70.0 },
    )
    .unwrap();
  let drag = state
    .text_input(TextAction::Drag, Point { x: 1940.0, y: 70.0 })
    .unwrap();
  assert!(drag
    .snapshot
    .unwrap()
    .rects
    .iter()
    .any(|rect| rect.kind == super::super::visual::VisualKind::Selection));
  state
    .text_input(TextAction::Up, Point { x: 1940.0, y: 70.0 })
    .unwrap();
  assert_eq!(state.selection_text(false, false).as_deref(), Some("hello"));
  assert_eq!(state.all_text().as_deref(), Some("hello"));
}

#[test]
fn qr_activates_only_after_press_and_release_on_the_same_code() {
  let state = state_with_qr();
  let point = Point {
    x: 1920.0,
    y: 110.0,
  };

  let hover = state.text_input(TextAction::Hover, point).unwrap();
  assert!(hover.qr_cursor);
  assert!(!hover.text_cursor);
  assert!(hover.qr_code.is_none());

  state
    .text_input(
      TextAction::Down {
        additive: false,
        double: false,
      },
      point,
    )
    .unwrap();
  let released = state.text_input(TextAction::Up, point).unwrap();
  assert_eq!(
    released.qr_code.map(|code| code.content).as_deref(),
    Some("https://screenwide.app")
  );
}

#[test]
fn dragging_away_cancels_qr_activation() {
  let state = state_with_qr();
  state
    .text_input(
      TextAction::Down {
        additive: false,
        double: false,
      },
      Point {
        x: 1920.0,
        y: 110.0,
      },
    )
    .unwrap();
  state
    .text_input(TextAction::Drag, Point { x: 1750.0, y: 60.0 })
    .unwrap();
  let released = state
    .text_input(
      TextAction::Up,
      Point {
        x: 1920.0,
        y: 110.0,
      },
    )
    .unwrap();
  assert!(released.qr_code.is_none());
}
