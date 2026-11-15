// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod tests;

use screenwide_osc::geometry::Point;
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::{
  tables::Copied, text_selection::TextSelection, visual::VisualSnapshot, TextRecognitionResult,
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
  pub copied: Option<Copied>,
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
    let copied = (action == TextAction::Copy)
      .then(|| session.text.as_ref().and_then(|model| copied(model, false)))
      .flatten();
    let snapshot = changed.then(|| {
      let model = session.text.as_ref().expect("text model remains installed");
      super::visual::snapshot(selection, model.result(), &model.rectangles())
    });
    Some(TextUpdate {
      snapshot,
      copied,
      qr_code,
      qr_cursor: qr_index.is_some(),
      text_cursor: inside && has_text && qr_index.is_none(),
    })
  }

  /// What copying puts on the clipboard: the selected text, or all of it
  /// when `fallback_all` and nothing is selected, joined into one paragraph
  /// with `paragraph`, and otherwise with its tables kept as tables.
  pub(super) fn copied(&self, fallback_all: bool, paragraph: bool) -> Option<Copied> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let model = session.text.as_ref()?;
    if paragraph {
      let selected = model.selected_text();
      let text = if selected.is_empty() && fallback_all {
        model.all_text().to_owned()
      } else {
        selected
      };
      return (!text.is_empty()).then(|| Copied {
        text: super::text_selection::paragraph(&text),
        html: None,
      });
    }
    copied(model, fallback_all)
  }
}

/// What copying the selection in `model` puts on the clipboard, or all it
/// read when `fallback_all` and nothing is selected.
fn copied(model: &TextSelection, fallback_all: bool) -> Option<Copied> {
  let selected = model.selected_sources();
  let copied = if selected.iter().any(|source| source.start < source.end) {
    super::tables::copied(model.result(), Some(&selected))
  } else if fallback_all {
    super::tables::copied(model.result(), None)
  } else {
    return None;
  };
  (!copied.text.is_empty()).then_some(copied)
}

/// Puts `copied` on the clipboard: as HTML, with the text for apps that take
/// only text, where it holds a table.
pub(super) fn write(app: &AppHandle, copied: Copied) -> Result<(), String> {
  let clipboard = app.clipboard();
  match copied.html {
    Some(html) => clipboard.write_html(html, Some(copied.text)),
    None => clipboard.write_text(copied.text),
  }
  .map_err(|error| error.to_string())
}

/// Copies the selected text, or all of it when `fallback_all` and nothing is
/// selected, and closes Text Recognition.
pub(crate) fn copy_selection_and_dismiss(
  app: &AppHandle,
  paragraph: bool,
  fallback_all: bool,
) -> Result<(), String> {
  let copied = app
    .state::<TextRecognitionState>()
    .copied(fallback_all, paragraph)
    .ok_or_else(|| "No recognized text is selected".to_owned())?;
  write(app, copied)?;
  super::dismiss(app);
  Ok(())
}
