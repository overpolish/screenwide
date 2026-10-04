// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A pasted or dropped picture, placed as a sticker without a hand on the
//! picture.

use tauri::AppHandle;

use super::super::preview_platform::SelectionGesturePhase;
use super::annotation_gesture::AnnotationCommit;
use super::state::{PreviewManager, ScreenshotPreviewState};
use crate::editor::annotations::gesture::{AnnotationGestureTarget, MODE_STICKER};
use crate::editor::annotations::sticker::StickerArt;
use crate::editor::stickers::dropped::{annotation_ids, fresh_id, StickerPoint};

impl PreviewManager {
  /// Places a sticker showing `art` `at` a point on a layer, or in the
  /// middle of the layer in hand, as a press with the sticker tool there
  /// would, whatever tool is in hand: it is dressed the same way, and
  /// committed as one edit. Unlike a sticker placed by a press, it is left
  /// in hand, as a pasted or dropped picture is something to dress at once.
  /// Nothing is placed while a gesture or a box's typing holds the layer.
  fn place_sticker(
    &mut self,
    art: StickerArt,
    at: Option<StickerPoint>,
  ) -> Option<AnnotationCommit> {
    if self.annotation_gesture.is_some() {
      return None;
    }
    let (pane, x, y) = at.map_or_else(
      || (self.annotation_pane_index.unwrap_or(0), 0.5, 0.5),
      |at| (at.layer, at.x, at.y),
    );
    let image_points = self.annotation_image_width(pane)?;
    let before = annotation_ids(self.annotations_for(pane)?);
    let mode = std::mem::replace(&mut self.annotation_mode, MODE_STICKER);
    let sticker = self.annotation_fresh.sticker.replace(art);
    let target = AnnotationGestureTarget::New;
    let press = |manager: &mut Self, phase| {
      manager.handle_annotation_gesture(phase, pane, target, x, y, 0, image_points)
    };
    let mut commit = press(self, SelectionGesturePhase::Begin)
      .is_none()
      .then(|| press(self, SelectionGesturePhase::End))
      .flatten();
    self.annotation_mode = mode;
    self.annotation_fresh.sticker = sticker;
    if let Some(commit) = commit.as_mut() {
      commit.selected_annotation_ids = fresh_id(&before, &commit.annotations).into_iter().collect();
      commit.selects_layer = self.annotation_pane_index != Some(pane);
    }
    // The grips are drawn for the tool in hand again, not the one borrowed.
    let chosen = commit
      .as_ref()
      .map(|commit| commit.selected_annotation_ids.clone())
      .unwrap_or_default();
    self.present_annotation_gesture(pane, &chosen);
    commit
  }
}

/// Places a sticker showing `art` on the open screenshot, `at` the point a
/// drop landed on or, for a paste, in the middle of the layer in hand, and
/// leaves it in hand. The screenshot is whichever one the preview has
/// open: there is only ever one, and the paste or drop that asks has only
/// just arrived.
#[tauri::command]
pub async fn place_screenshot_sticker(
  app: AppHandle,
  state: tauri::State<'_, ScreenshotPreviewState>,
  art: StickerArt,
  at: Option<StickerPoint>,
) -> Result<bool, String> {
  let (session_id, commit) = {
    let mut manager = state
      .0
      .lock()
      .map_err(|_| "The screenshot preview is unavailable")?;
    let Some(session_id) = manager.session_id else {
      return Ok(false);
    };
    (session_id, manager.place_sticker(art, at))
  };
  let placed = commit.is_some();
  if let Some(commit) = commit {
    super::start_callbacks::emit_annotation_change(&app, session_id, commit);
  }
  Ok(placed)
}
