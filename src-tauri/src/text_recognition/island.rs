// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while Text Recognition is up. It comes out at the top of the
//! screen while an area is drawn, then hangs by the area while it is read
//! and holds what can be done with what was found, or what a QR code pressed
//! in it holds. It stands above the overlay, so the overlay never takes a
//! press meant for it.
//!
//! The island shows the session rather than keeping its own state: each
//! change is sent (`text-recognition://island`), and only a change.

use std::sync::PoisonError;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use screenwide_osc::{desktop::DesktopBinding, geometry::Rect};

use super::{RecognizedQrCode, TextRecognitionState};
use crate::app_windows;
use crate::app_windows::island::{IslandClaim, IslandSlot};
use crate::app_windows::island_placement::{IslandPlacement, LocalRect};

/// What the island shows of a Text Recognition session.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "phase", rename_all = "camelCase")]
#[ts(export)]
pub enum TextRecognitionIslandState {
  /// An area is being drawn; `failed` once reading the last one failed.
  Choosing { failed: bool },
  /// An area is being read; `loading` while the models that read it load.
  Reading { loading: bool },
  /// What reading found, and whether any of the text is selected.
  Ready {
    lines: u32,
    #[serde(rename = "qrCodes")]
    qr_codes: u32,
    selected: bool,
  },
  /// A QR code pressed in what was found: what it holds, or why it could not
  /// be read.
  Qr {
    content: String,
    #[serde(rename = "decodeError")]
    decode_error: Option<String>,
  },
}

/// What a press on the island does.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TextRecognitionIslandAction {
  /// The selected text, or all of it with none selected, its tables kept as
  /// tables.
  Copy,
  /// The same, joined into one paragraph.
  CopyParagraph,
  SelectAnotherArea,
  /// Back from a QR code's contents to what was found.
  CloseQrCode,
  Close,
}

/// What the island was last told; none while Text Recognition is not on it.
static ISLAND: IslandSlot<TextRecognitionIslandState> =
  IslandSlot::new(IslandClaim::TextRecognition, "text-recognition://island");

/// Text Recognition is starting: the island comes out at the top at once, so
/// the start shows while the desktop freezes. Called with the island kept
/// out of captures.
pub(super) fn come_out(app: &AppHandle) {
  ISLAND.tell(app, TextRecognitionIslandState::Choosing { failed: false });
}

/// The overlay is presented: the island stands above it.
pub(super) fn stand_over(app: &AppHandle, overlay: &tauri::WebviewWindow) {
  if !ISLAND.is_up() {
    return;
  }
  app_windows::island_overlay::stand_over(app, Some(overlay));
}

/// An area was drawn, in the anchor display's points: the island hangs by it
/// while it is read, on the display that holds most of it.
pub(super) fn reading(app: &AppHandle, binding: &DesktopBinding, region: Rect) {
  let state = TextRecognitionIslandState::Reading {
    loading: model_loading(),
  };
  match local_box(binding, region) {
    Some((display, rect)) => ISLAND.tell_at(app, state, IslandPlacement::ByBox { display, rect }),
    None => ISLAND.update(app, state),
  }
}

/// Whether the models that read text are still loading, which a read then
/// waits for.
fn model_loading() -> bool {
  #[cfg(target_os = "windows")]
  return super::windows::reader::is_loading();
  #[cfg(target_os = "macos")]
  return super::macos::warm_up::is_running();
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  false
}

/// The models that read text have loaded: a read waiting for them goes on.
pub(super) fn model_loaded(app: &AppHandle) {
  if ISLAND.current() == Some(TextRecognitionIslandState::Reading { loading: true }) {
    ISLAND.update(app, TextRecognitionIslandState::Reading { loading: false });
  }
}

/// The part of `region`, in the anchor display's points, on the display that
/// holds most of it, in that display's points. The displays' origins are on
/// the desktop, which starts at its top start corner rather than at the
/// anchor's.
fn local_box(binding: &DesktopBinding, region: Rect) -> Option<(u32, LocalRect)> {
  let region = binding.project_local(region)?;
  binding
    .displays
    .iter()
    .filter_map(|display| {
      let left = region.origin.x.max(display.origin.x);
      let top = region.origin.y.max(display.origin.y);
      let right = region.right().min(display.origin.x + display.size.width);
      let bottom = region.bottom().min(display.origin.y + display.size.height);
      (right > left && bottom > top).then(|| {
        let rect = LocalRect {
          x: left - display.origin.x,
          y: top - display.origin.y,
          width: right - left,
          height: bottom - top,
        };
        (display.id, rect)
      })
    })
    .max_by(|(_, a), (_, b)| (a.width * a.height).total_cmp(&(b.width * b.height)))
}

/// Reading finished, or the selection in what it found changed.
pub(super) fn sync_ready(app: &AppHandle) {
  if let Some(state) = app.state::<TextRecognitionState>().island_state() {
    ISLAND.update(app, state);
  }
}

/// A QR code was pressed: the island says what it holds, in place of what
/// was found.
pub(super) fn show_qr_code(app: &AppHandle, code: RecognizedQrCode) {
  ISLAND.update(
    app,
    TextRecognitionIslandState::Qr {
      content: code.content,
      decode_error: code.decode_error,
    },
  );
}

/// A press on the overlay away from the QR codes, or Back on the island,
/// puts a QR code's contents away and shows what was found again.
pub(super) fn close_qr_code(app: &AppHandle) {
  if matches!(
    ISLAND.current(),
    Some(TextRecognitionIslandState::Qr { .. })
  ) {
    sync_ready(app);
  }
}

/// The overlay takes a new area, after reading one failed or to read
/// another, and the island goes back to the top of the screen to say so.
pub(super) fn choosing(app: &AppHandle, failed: bool) {
  if !ISLAND.is_up() {
    return;
  }
  ISLAND.tell_at(
    app,
    TextRecognitionIslandState::Choosing { failed },
    IslandPlacement::Top,
  );
}

/// Text Recognition went. The island drops it and goes back to whatever
/// else holds it, or tucks away. It stops standing over the overlay first,
/// as on Windows a window takes those it owns with it when it closes.
pub(super) fn withdraw(app: &AppHandle) {
  if !ISLAND.is_up() {
    return;
  }
  app_windows::island_overlay::stand_over(app, None);
  ISLAND.withdraw(app);
}

impl TextRecognitionState {
  /// What reading found, as the island shows it; none before it has.
  pub(super) fn island_state(&self) -> Option<TextRecognitionIslandState> {
    let session = self.0.lock().unwrap_or_else(PoisonError::into_inner);
    let model = session.text.as_ref()?;
    let result = model.result();
    Some(TextRecognitionIslandState::Ready {
      lines: u32::try_from(result.lines.len()).unwrap_or(u32::MAX),
      qr_codes: u32::try_from(result.qr_codes.len()).unwrap_or(u32::MAX),
      selected: !model.selected_text().is_empty(),
    })
  }
}

#[tauri::command]
pub fn text_recognition_island_state() -> Option<TextRecognitionIslandState> {
  ISLAND.current()
}

#[tauri::command]
pub fn text_recognition_island_action(app: AppHandle, action: TextRecognitionIslandAction) {
  tauri::async_runtime::spawn(async move {
    let result = match action {
      TextRecognitionIslandAction::Copy => super::copy_selection_and_dismiss(&app, false, true),
      TextRecognitionIslandAction::CopyParagraph => {
        super::copy_selection_and_dismiss(&app, true, true)
      }
      TextRecognitionIslandAction::SelectAnotherArea => {
        super::adapter::choose_again(&app);
        Ok(())
      }
      TextRecognitionIslandAction::CloseQrCode => {
        close_qr_code(&app);
        Ok(())
      }
      TextRecognitionIslandAction::Close => {
        super::dismiss(&app);
        Ok(())
      }
    };
    if let Err(error) = result {
      eprintln!("Could not run {action:?} from the Text Recognition island: {error}");
    }
  });
}

#[cfg(test)]
mod tests;
