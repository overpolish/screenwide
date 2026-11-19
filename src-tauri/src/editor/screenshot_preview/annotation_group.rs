// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Several annotations chosen together on a still's layer, carried on the
//! picture as one. The manager owns the layer's working copy through the
//! carry, as it does through any annotation gesture, and the release hands
//! the whole list back to React as one edit.

use super::annotation_gesture::AnnotationCommit;
use super::state::PreviewManager;
use screenwide_annotations::copy::copy_in_place;
use screenwide_annotations::group::{group_bounds, GroupMember, GroupMove};
use screenwide_annotations::handles::source_point;
use screenwide_annotations::snap::{
  source_per_size, threshold_source_px, SnapField, SnapModifiers, SnapRequest, SnapResult,
};
use screenwide_annotations::Annotation;
use screenwide_workspace::editing::SelectionGesturePhase;

pub(crate) struct GroupDrag {
  pane_index: u32,
  /// The layer's annotations and the choice as the carry found them, which
  /// a cancel puts back, taking any copies it made with it.
  before: Vec<Annotation>,
  before_selected: Vec<String>,
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
      self.annotation_selected = drag.before_selected;
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
      let duplicate = SnapModifiers::from_bits(snap).duplicate;
      self.annotation_group = Some(self.begin_group(pane_index, x, y, duplicate)?);
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

  /// The carry of the chosen group, from where it was pressed. With the
  /// duplicate modifier it takes copies of them, which move while the
  /// originals stay, and the copies are chosen in their place.
  fn begin_group(&mut self, pane_index: u32, x: f64, y: f64, duplicate: bool) -> Option<GroupDrag> {
    let source = self.annotation_source(pane_index)?;
    let image_width = self.annotation_image_width(pane_index).unwrap_or_default();
    let before = self.annotations_for(pane_index)?.clone();
    let before_selected = self.annotation_selected.clone();
    if duplicate {
      let copied = copy_in_place(self.pane_annotations_mut(pane_index)?, &before_selected);
      self.annotation_selected = copied.into_iter().map(|copied| copied.copy).collect();
    }
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
      before,
      before_selected,
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
