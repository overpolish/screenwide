// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A pasted or dropped picture, placed as a sticker without a hand on the
//! picture.

use super::*;
use crate::editor::annotations::gesture::MODE_STICKER;
use crate::editor::annotations::sticker::StickerArt;
use crate::editor::stickers::dropped::{annotation_ids, fresh_id, StickerPoint};

impl PreviewPlayerManager {
  /// Places a sticker showing `art` `at` a point on a pane, or in the middle
  /// of the pane in hand, as a press with the sticker tool there would,
  /// whatever tool is in hand: it is timed and dressed the same way, and
  /// committed as one edit. Unlike a sticker placed by a press, it is left
  /// in hand, as a pasted or dropped picture is something to dress at once.
  /// Nothing is placed while playing or while a box is being typed into.
  pub(super) fn place_sticker(
    &mut self,
    art: StickerArt,
    at: Option<StickerPoint>,
  ) -> Option<gesture::Commit> {
    if self.is_playing || self.annotation.gesture.is_some() {
      return None;
    }
    let (pane, x, y) = at.map_or_else(
      || (self.annotation.pane.unwrap_or(0), 0.5, 0.5),
      |at| (at.layer, at.x, at.y),
    );
    let image_points = self.pane_image_widths(pane).1;
    let before = annotation_ids(&self.pane_annotations(pane));
    let mode = std::mem::replace(&mut self.annotation.mode, MODE_STICKER);
    let sticker = self.annotation.fresh.sticker.replace(art);
    let target = AnnotationGestureTarget::New;
    let press = |manager: &mut Self, phase| {
      manager.annotation_gesture(phase, pane, target, x, y, 0, image_points)
    };
    let mut commit = press(self, SelectionGesturePhase::Begin)
      .is_none()
      .then(|| press(self, SelectionGesturePhase::End))
      .flatten();
    self.annotation.mode = mode;
    self.annotation.fresh.sticker = sticker;
    if let Some(commit) = commit.as_mut() {
      commit.selected_annotation_ids = fresh_id(&before, &commit.annotations).into_iter().collect();
      self.annotation.selected = commit.selected_annotation_ids.clone();
    }
    // The grips are drawn for the tool in hand again, not the one borrowed.
    self.publish_annotation_handles();
    commit
  }
}

/// Places a sticker showing `art` on the open recording, `at` the point a
/// drop landed on or, for a paste, in the middle of the pane in hand, and
/// leaves it in hand. The recording is whichever one the preview has open:
/// there is only ever one, and the paste or drop that asks has only just
/// arrived.
#[tauri::command]
pub async fn place_recording_sticker(
  app: AppHandle,
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  art: StickerArt,
  at: Option<StickerPoint>,
) -> Result<bool, String> {
  let commit = {
    let mut manager = state
      .0
      .lock()
      .map_err(|_| "The recording preview is unavailable")?;
    if manager.session_id.is_none() {
      return Ok(false);
    }
    manager.place_sticker(art, at)
  };
  let placed = commit.is_some();
  if let Some(commit) = commit {
    let _ = app.emit("editor://recording-annotations", commit);
  }
  Ok(placed)
}
