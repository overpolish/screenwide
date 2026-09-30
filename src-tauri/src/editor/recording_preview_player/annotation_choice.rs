// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two ways a press on the picture changes the choice without editing
//! anything: a click with the toggle modifier, which adds one annotation or
//! takes it out, and a marquee band, which chooses every annotation it
//! touches on the pane it was drawn over.

use super::gesture::Commit;
use super::*;
use crate::editor::annotations::marquee::{chosen_after_band, swept_annotations};
use crate::editor::annotations::AnnotationPoint;

/// A marquee band under way: the pane it was pressed on, the corner it was
/// pressed at, in that pane's source pixels, and whether it adds to the
/// choice rather than replacing it.
pub(super) struct Band {
  pane: u32,
  from: AnnotationPoint,
  additive: bool,
}

impl PreviewPlayerManager {
  /// A click with the toggle modifier on the annotation at `index` of what
  /// `pane` shows: it joins the choice, or leaves it.
  pub(super) fn toggle_annotation(
    &mut self,
    pane: u32,
    index: usize,
    position: u64,
  ) -> Option<Commit> {
    let shown = self.pane_annotations(pane);
    let id = shown.get(index)?.id.clone();
    let chosen = &mut self.annotation.selected;
    match chosen.iter().position(|item| *item == id) {
      Some(at) => {
        chosen.remove(at);
      }
      None => chosen.push(id),
    }
    self.commit_choice(pane, position, shown)
  }

  /// One end of a marquee band drawn over `pane`: the press stores where it
  /// began, and the release chooses what the band touched there, leaving
  /// pinned annotations out. The positional modifier held at the press adds
  /// to the choice.
  pub(super) fn marquee_gesture(
    &mut self,
    phase: SelectionGesturePhase,
    pane: u32,
    x: f64,
    y: f64,
    snap: u32,
  ) -> Option<Commit> {
    let source = self.pane_source(pane)?;
    let point = source_point(x, y, source);
    match phase {
      SelectionGesturePhase::Begin => {
        self.annotation.band = Some(Band {
          pane,
          from: point,
          additive: SnapModifiers::from_bits(snap).position,
        });
        None
      }
      SelectionGesturePhase::Update => None,
      SelectionGesturePhase::Cancel => {
        self.annotation.band = None;
        None
      }
      SelectionGesturePhase::End => {
        let band = self
          .annotation
          .band
          .take()
          .filter(|band| band.pane == pane)?;
        let shown = self.pane_annotations(pane);
        let pinned: Vec<String> = self
          .sources
          .as_ref()?
          .annotation_clips
          .read()
          .ok()?
          .iter()
          .filter(|clip| clip.pin.is_some())
          .map(|clip| clip.annotation.id.clone())
          .collect();
        let swept = swept_annotations(
          &shown,
          (band.from, point),
          source_per_size(source, self.pane_image_widths(pane).1),
          |annotation| !pinned.contains(&annotation.id),
        );
        // Command held at either end adds: it may be taken up part way
        // through the band, as it may through any other drag.
        let additive = band.additive || SnapModifiers::from_bits(snap).position;
        self.annotation.selected = chosen_after_band(&self.annotation.selected, swept, additive);
        let position = self
          .position_ms
          .min(self.sources.as_ref()?.duration_ms.saturating_sub(1));
        self.commit_choice(pane, position, shown)
      }
    }
  }

  /// Shows the choice as it now stands and tells React, with the pane's
  /// annotations as they were: a choice, not an edit.
  fn commit_choice(&mut self, pane: u32, position: u64, shown: Vec<Annotation>) -> Option<Commit> {
    self.publish_annotation_handles();
    #[cfg(target_os = "macos")]
    if let Some(surface) = self
      .sources
      .as_ref()
      .and_then(|sources| sources.preview_surface.as_ref())
    {
      surface.redraw_recording_workspace();
    }
    self.commit(
      pane,
      position,
      shown,
      self.annotation.selected.clone(),
      None,
    )
  }
}
