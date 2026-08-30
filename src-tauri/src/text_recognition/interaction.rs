// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::osc::geometry::Point;
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::{
  text_selection::TextSelection, visual::VisualSnapshot, TextRecognitionResult,
  TextRecognitionState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAction {
  Hover,
  Down { additive: bool, double: bool },
  Drag,
  Up,
  SelectAll,
  Copy,
}

pub struct TextUpdate {
  pub snapshot: Option<VisualSnapshot>,
  pub copy_text: Option<String>,
  pub qr_code: Option<super::RecognizedQrCode>,
  pub qr_cursor: bool,
  pub text_cursor: bool,
}

impl TextRecognitionState {
  pub(super) fn install_result(&self, generation: u64, result: TextRecognitionResult) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.generation != generation || session.selection.is_none() {
      return false;
    }
    session.text = Some(TextSelection::new(result));
    session.pressed_qr = None;
    true
  }

  pub(super) fn text_input(&self, action: TextAction, desktop_point: Point) -> Option<TextUpdate> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let selection = session.selection?;
    let inside = selection.contains(desktop_point);
    let point = Point {
      x: (desktop_point.x - selection.origin.x) / selection.size.width,
      y: (desktop_point.y - selection.origin.y) / selection.size.height,
    };
    let qr_index = if inside {
      session
        .text
        .as_ref()?
        .result()
        .qr_codes
        .iter()
        .position(|code| {
          point.x >= code.bounds.x
            && point.y >= code.bounds.y
            && point.x <= code.bounds.x + code.bounds.width
            && point.y <= code.bounds.y + code.bounds.height
        })
    } else {
      None
    };
    let has_text = !session.text.as_ref()?.result().lines.is_empty();
    let mut qr_code = None;
    let changed = match action {
      TextAction::Hover | TextAction::Copy => false,
      TextAction::Down { .. } if qr_index.is_some() => {
        session.pressed_qr = qr_index;
        false
      }
      TextAction::Down { additive, double } if inside => {
        session.pressed_qr = None;
        session.text.as_mut()?.pointer_down(point, additive, double)
      }
      TextAction::Down { .. } => {
        session.pressed_qr = None;
        false
      }
      TextAction::Drag if session.pressed_qr.is_some() => {
        if session.pressed_qr != qr_index {
          session.pressed_qr = None;
        }
        false
      }
      TextAction::Drag => session.text.as_mut()?.pointer_move(point),
      TextAction::Up if session.pressed_qr.is_some() => {
        let pressed = session.pressed_qr.take();
        if pressed == qr_index {
          qr_code =
            pressed.and_then(|index| session.text.as_ref()?.result().qr_codes.get(index).cloned());
        }
        false
      }
      TextAction::Up => session.text.as_mut()?.pointer_up(point),
      TextAction::SelectAll => session.text.as_mut()?.select_all(),
    };
    let copy_text = (action == TextAction::Copy)
      .then(|| session.text.as_ref().map(|model| model.selected_text()))
      .flatten()
      .filter(|text| !text.is_empty());
    let snapshot = changed.then(|| {
      let model = session.text.as_ref().expect("text model remains installed");
      super::visual::snapshot(selection, model.result(), &model.rectangles())
    });
    Some(TextUpdate {
      snapshot,
      copy_text,
      qr_code,
      qr_cursor: qr_index.is_some(),
      text_cursor: inside && has_text && qr_index.is_none(),
    })
  }

  pub(super) fn selection_text(&self, fallback_all: bool, paragraph: bool) -> Option<String> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let model = session.text.as_ref()?;
    let selected = model.selected_text();
    let text = if selected.is_empty() && fallback_all {
      model.all_text().to_owned()
    } else {
      selected
    };
    (!text.is_empty()).then(|| {
      if paragraph {
        super::text_selection::paragraph(&text)
      } else {
        text
      }
    })
  }

  pub(super) fn all_text(&self) -> Option<String> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session
      .text
      .as_ref()
      .map(|model| model.all_text().to_owned())
      .filter(|text| !text.is_empty())
  }
}

pub(crate) fn copy_selection_and_dismiss(
  app: &AppHandle,
  paragraph: bool,
  fallback_all: bool,
) -> Result<(), String> {
  let text = app
    .state::<TextRecognitionState>()
    .selection_text(fallback_all, paragraph)
    .ok_or_else(|| "No recognized text is selected".to_owned())?;
  app
    .clipboard()
    .write_text(text)
    .map_err(|error| error.to_string())?;
  super::dismiss(app);
  Ok(())
}

pub(crate) fn copy_all_and_dismiss(app: &AppHandle) -> Result<(), String> {
  let text = app
    .state::<TextRecognitionState>()
    .all_text()
    .ok_or_else(|| "No recognized text is available".to_owned())?;
  app
    .clipboard()
    .write_text(text)
    .map_err(|error| error.to_string())?;
  super::dismiss(app);
  Ok(())
}

#[cfg(test)]
mod tests {
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
}
