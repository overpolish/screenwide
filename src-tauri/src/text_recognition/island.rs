// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while Text Recognition is up. It comes out at the top of the
//! screen while an area is drawn, then hangs by the area while it is read
//! and holds what can be done with what was found. It stands above the
//! overlay, so the overlay never takes a press meant for it.
//!
//! The island shows the session rather than keeping its own state: each
//! change is sent (`text-recognition://island`), and only a change.

use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use screenwide_osc::{desktop::DesktopBinding, geometry::Rect};

use super::TextRecognitionState;
use crate::app_windows::island_placement::{IslandPlacement, LocalRect};
use crate::app_windows::{self, WindowLabel};

const EVENT: &str = "text-recognition://island";

/// What the island shows of a Text Recognition session.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "phase", rename_all = "camelCase")]
#[ts(export)]
pub enum TextRecognitionIslandState {
  /// An area is being drawn; `failed` once reading the last one failed.
  Choosing {
    failed: bool,
  },
  Reading,
  /// What reading found, and whether any of the text is selected.
  Ready {
    lines: u32,
    #[serde(rename = "qrCodes")]
    qr_codes: u32,
    selected: bool,
  },
}

/// What a press on the island does.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TextRecognitionIslandAction {
  /// The selected text, or all of it with none selected.
  Copy,
  /// The same, joined into one paragraph.
  CopyParagraph,
  SelectAnotherArea,
  Close,
}

/// What the island was last told; none while Text Recognition is not on it.
static SHOWN: Mutex<Option<TextRecognitionIslandState>> = Mutex::new(None);

fn shown() -> std::sync::MutexGuard<'static, Option<TextRecognitionIslandState>> {
  SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}

fn tell(app: &AppHandle, state: Option<TextRecognitionIslandState>) {
  let mut shown = shown();
  if *shown == state {
    return;
  }
  *shown = state;
  drop(shown);
  let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, state);
}

fn place(app: &AppHandle, placement: IslandPlacement) {
  if let Err(error) = app_windows::island::place(app, placement) {
    eprintln!("Could not place the island for Text Recognition: {error}");
  }
}

/// Text Recognition is up: the island comes out at the top of the screen,
/// above the overlay, while an area is drawn. Called once the overlay is
/// presented, so on Windows the island is owned by it.
pub(super) fn present(app: &AppHandle, overlay: &tauri::WebviewWindow) {
  tell(
    app,
    Some(TextRecognitionIslandState::Choosing { failed: false }),
  );
  app_windows::island_overlay::stand_over(app, Some(overlay));
  place(app, IslandPlacement::Top);
  if let Err(error) = app_windows::island::show(app) {
    eprintln!("Could not show the island for Text Recognition: {error}");
  }
}

/// An area was drawn: the island hangs by it while it is read, on the
/// display that holds most of it.
pub(super) fn reading(app: &AppHandle, binding: &DesktopBinding, region: Rect) {
  tell(app, Some(TextRecognitionIslandState::Reading));
  let Some((display, rect)) = local_box(binding, region) else {
    return;
  };
  place(app, IslandPlacement::ByBox { display, rect });
}

/// The part of `region` on the display that holds most of it, in that
/// display's points.
fn local_box(binding: &DesktopBinding, region: Rect) -> Option<(u32, LocalRect)> {
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
  if shown().is_none() {
    return;
  }
  if let Some(state) = app.state::<TextRecognitionState>().island_state() {
    tell(app, Some(state));
  }
}

/// The overlay takes a new area, after reading one failed or to read
/// another, and the island goes back to the top of the screen to say so.
pub(super) fn choosing(app: &AppHandle, failed: bool) {
  if shown().is_none() {
    return;
  }
  tell(app, Some(TextRecognitionIslandState::Choosing { failed }));
  place(app, IslandPlacement::Top);
}

/// Text Recognition went. The island drops it and goes, tucking away where
/// it is, unless the recording or its controls keep it; then it hands over
/// to them, coming out again at the top with what they show. It stops
/// standing over the overlay first, as on Windows a window takes those it
/// owns with it when it closes.
pub(super) fn withdraw(app: &AppHandle) {
  if shown().is_none() {
    return;
  }
  tell(app, None);
  app_windows::island_overlay::stand_over(app, None);
  let result = if crate::recording::is_idle(app) && !app_windows::is_recording_ui_visible() {
    app_windows::island::hide(app)
      .and_then(|()| app_windows::island::place(app, IslandPlacement::Top))
  } else {
    app_windows::island::hand_over(app, IslandPlacement::Top)
  };
  if let Err(error) = result {
    eprintln!("Could not put the island away after Text Recognition: {error}");
  }
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
  *shown()
}

#[tauri::command]
pub fn text_recognition_island_action(app: AppHandle, action: TextRecognitionIslandAction) {
  super::qr_details::hide_without_resume(&app);
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
mod tests {
  use screenwide_osc::desktop::DesktopDisplay;
  use screenwide_osc::geometry::{Point, Size};

  use super::*;

  fn display(id: u32, x: f64) -> DesktopDisplay {
    DesktopDisplay {
      id,
      origin: Point { x, y: 0.0 },
      size: Size {
        width: 100.0,
        height: 80.0,
      },
      scale: 2.0,
    }
  }

  #[test]
  fn an_area_across_two_displays_belongs_to_the_one_holding_most_of_it() {
    let binding = DesktopBinding {
      displays: vec![display(1, 0.0), display(2, 100.0)],
      anchor_id: 1,
      size: Size {
        width: 200.0,
        height: 80.0,
      },
      layout_changed: false,
    };
    let (id, rect) = local_box(&binding, Rect::from_xywh(90.0, 10.0, 40.0, 20.0)).unwrap();
    assert_eq!(id, 2);
    assert_eq!(
      rect,
      LocalRect {
        x: 0.0,
        y: 10.0,
        width: 30.0,
        height: 20.0,
      }
    );
  }
}
