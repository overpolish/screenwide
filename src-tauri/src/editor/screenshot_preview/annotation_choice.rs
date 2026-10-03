// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two ways a press on a still changes the choice without editing
//! anything: a click with the toggle modifier, which adds one annotation or
//! takes it out, and a marquee band, which chooses every annotation it
//! touches. A still's choice belongs to one layer, so a choice made on
//! another layer starts that layer's afresh and selects it.

use super::super::preview_platform::SelectionGesturePhase;
use super::annotation_gesture::AnnotationCommit;
use super::state::PreviewManager;
use crate::editor::annotations::handles::source_point;
use crate::editor::annotations::marquee::{chosen_after_band, swept_annotations};
use crate::editor::annotations::snap::{source_per_size, SnapModifiers};
use crate::editor::annotations::AnnotationPoint;

/// A marquee band under way: the layer it was pressed on, the corner it was
/// pressed at, in that layer's source pixels, and whether it adds to the
/// choice rather than replacing it.
pub(crate) struct Band {
  pane_index: u32,
  from: AnnotationPoint,
  additive: bool,
}

impl PreviewManager {
  /// A click with the toggle modifier on the annotation at `index`: it joins
  /// the choice, or leaves it.
  pub(super) fn toggle_annotation(
    &mut self,
    pane_index: u32,
    index: usize,
  ) -> Option<AnnotationCommit> {
    let id = self.annotations_for(pane_index)?.get(index)?.id.clone();
    let mut chosen = self.choice_on(pane_index);
    match chosen.iter().position(|item| *item == id) {
      Some(at) => {
        chosen.remove(at);
      }
      None => chosen.push(id),
    }
    self.annotation_selected = chosen;
    self.commit_choice(pane_index)
  }

  /// One end of a marquee band: the press stores where it began, and the
  /// release chooses what the band touched. The positional modifier held at
  /// the press adds to the choice.
  pub(super) fn marquee_annotation_gesture(
    &mut self,
    phase: SelectionGesturePhase,
    pane_index: u32,
    x: f64,
    y: f64,
    snap: u32,
  ) -> Option<AnnotationCommit> {
    let source = self.annotation_source(pane_index)?;
    let point = source_point(x, y, source);
    match phase {
      SelectionGesturePhase::Begin => {
        self.annotation_band = Some(Band {
          pane_index,
          from: point,
          additive: SnapModifiers::from_bits(snap).position,
        });
        None
      }
      SelectionGesturePhase::Update => None,
      SelectionGesturePhase::Cancel => {
        self.annotation_band = None;
        None
      }
      SelectionGesturePhase::End => {
        let band = self
          .annotation_band
          .take()
          .filter(|band| band.pane_index == pane_index)?;
        let image_width = self.annotation_image_width(pane_index).unwrap_or_default();
        let swept = swept_annotations(
          self.annotations_for(pane_index)?,
          (band.from, point),
          source_per_size(source, image_width),
          |_| true,
        );
        // Command held at either end adds: it may be taken up part way
        // through the band, as it may through any other drag.
        let additive = band.additive || SnapModifiers::from_bits(snap).position;
        self.annotation_selected = chosen_after_band(&self.choice_on(pane_index), swept, additive);
        self.commit_choice(pane_index)
      }
    }
  }

  /// Shows the choice as it now stands and tells React, with the layer's list
  /// unchanged: a choice, not an edit.
  fn commit_choice(&mut self, pane_index: u32) -> Option<AnnotationCommit> {
    let chosen = self.annotation_selected.clone();
    self.present_annotation_gesture(pane_index, &chosen);
    self.commit_for(pane_index, chosen)
  }

  /// The choice a press on `pane_index` adds to or takes from: the standing
  /// one on the selected layer, and nothing on any other.
  fn choice_on(&self, pane_index: u32) -> Vec<String> {
    if self.annotation_pane_index == Some(pane_index) {
      self.annotation_selected.clone()
    } else {
      Vec::new()
    }
  }
}
