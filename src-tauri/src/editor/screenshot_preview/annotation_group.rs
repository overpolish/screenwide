// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Several annotations chosen together on a still's layer, carried on the
//! picture as one. The manager owns the layer's working copy through the
//! carry, as it does through any annotation gesture, and the release hands
//! the whole list back to React as one edit.

use super::super::preview_platform::SelectionGesturePhase;
use super::annotation_gesture::AnnotationCommit;
use super::state::PreviewManager;
use crate::editor::annotations::group::{group_bounds, GroupMember, GroupMove};
use crate::editor::annotations::handles::source_point;
use crate::editor::annotations::snap::{
  source_per_size, threshold_source_px, SnapField, SnapModifiers, SnapRequest, SnapResult,
};
use crate::editor::annotations::Annotation;

pub(crate) struct GroupDrag {
  pane_index: u32,
  before: Vec<Annotation>,
  moving: GroupMove,
  field: SnapField,
}

impl PreviewManager {
  /// One sample of the chosen group carried by a press on the layer.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn group_annotation_gesture(
    &mut self,
    phase: SelectionGesturePhase,
    pane_index: u32,
    x: f64,
    y: f64,
    snap: u32,
    image_points: f64,
  ) -> Option<AnnotationCommit> {
    if matches!(phase, SelectionGesturePhase::Cancel) {
      let drag = self.annotation_group.take()?;
      *self.pane_annotations_mut(drag.pane_index)? = drag.before;
      self.settle_group(drag.pane_index);
      return None;
    }
    if matches!(phase, SelectionGesturePhase::Begin) {
      if self.annotation_text.is_some() || self.annotation_selected.len() < 2 {
        return None;
      }
      // The chrome is drawn from React's latest layout, so the carry starts
      // from that same snapshot, as every annotation gesture does.
      let base = self.react_output.clone().or_else(|| self.output.clone())?;
      self.output = Some(base);
      self.annotation_group = Some(self.begin_group(pane_index, x, y)?);
      return None;
    }
    let drag = self.annotation_group.as_ref()?;
    let pane_index = drag.pane_index;
    let source = self.annotation_source(pane_index)?;
    let request = threshold_source_px(source.0, image_points).map(|threshold| SnapRequest {
      field: &drag.field,
      threshold,
    });
    let (moved, result) = drag.moving.update(
      source_point(x, y, source),
      SnapModifiers::from_bits(snap),
      request,
    );
    let annotations = self.pane_annotations_mut(pane_index)?;
    for annotation in moved {
      if let Some(slot) = annotations.iter_mut().find(|item| item.id == annotation.id) {
        *slot = annotation;
      }
    }
    let chosen = self.annotation_selected.clone();
    if !matches!(phase, SelectionGesturePhase::End) {
      self.publish_annotation_snap(pane_index, &result);
      self.present_annotation_gesture(pane_index, &chosen);
      return None;
    }
    self.annotation_group = None;
    self.settle_group(pane_index);
    self.commit_for(pane_index, chosen)
  }

  fn pane_annotations_mut(&mut self, pane_index: u32) -> Option<&mut Vec<Annotation>> {
    Some(
      &mut self
        .output
        .as_mut()?
        .items
        .get_mut(pane_index as usize)?
        .output
        .annotations,
    )
  }

  fn begin_group(&self, pane_index: u32, x: f64, y: f64) -> Option<GroupDrag> {
    let source = self.annotation_source(pane_index)?;
    let image_width = self.annotation_image_width(pane_index).unwrap_or_default();
    let annotations = self.annotations_for(pane_index)?;
    let members: Vec<GroupMember> = annotations
      .iter()
      .filter(|annotation| self.annotation_selected.contains(&annotation.id))
      .map(|annotation| GroupMember {
        before: annotation.clone(),
        scale: 1.0,
      })
      .collect();
    if members.is_empty() {
      return None;
    }
    let held = group_bounds(
      members.iter().map(|member| &member.before),
      source_per_size(source, image_width),
    );
    let moving = GroupMove::new(source_point(x, y, source), members, held);
    let field = SnapField::without(source, annotations, |id| moving.holds(id), image_width);
    Some(GroupDrag {
      pane_index,
      before: annotations.clone(),
      moving,
      field,
    })
  }

  /// Puts the snap chrome away and draws the layer as the carry left it.
  fn settle_group(&self, pane_index: u32) {
    self.publish_annotation_snap(pane_index, &SnapResult::default());
    self.present_annotation_gesture(pane_index, &self.annotation_selected);
  }
}
