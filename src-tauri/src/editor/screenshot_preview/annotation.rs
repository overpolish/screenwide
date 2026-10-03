// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Screenshot-specific annotation layout and hover state. Shared handles and
//! editing live in the editor's annotation module.

use crate::editor::annotations::group::{group_boxes, NativeAnnotationGroupBox};
use crate::editor::annotations::handles::{
  annotation_handles, NativeAnnotationHandles, HANDLE_FLAG_GROUPED,
};
use crate::editor::annotations::{Annotation, AnnotationShape, AnnotationStyle};

/// The tool modes and their mapping live with the gesture model, so the two
/// workspaces take the tool in hand the same way.
use crate::editor::annotations::gesture::{annotation_mode, hovers_nothing};

/// Where the hover halo is, and how wide. The width is already in the layer's
/// canvas pixels: the native side reports how big the picture is drawn, and
/// the manager knows how many canvas pixels that is. The halo is placed by
/// the annotation's index, and `id` says which annotation that was, so a
/// list that changes under it cannot hand the halo on to another.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnnotationHover {
  pub(crate) id: String,
  pub(crate) index: usize,
  pub(crate) layer_id: u64,
  pub(crate) width: f32,
}

/// The halo's width in display points over the pulse, eased the way the
/// ruler's is: three points growing to eight over 160 ms.
pub(crate) fn hover_width_points(progress: f64) -> f64 {
  let eased = 1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3);
  3.0 + (8.0 - 3.0) * eased
}

/// What one layout push tells the native OSC about this layer's arrows.
pub(super) struct AnnotationLayout {
  pub(super) handles: Vec<NativeAnnotationHandles>,
  /// The strokes' fitted lines the grips point into.
  pub(super) paths: Vec<[f32; 2]>,
  /// The arrow whose three grips are drawn, or -1 for none.
  pub(super) selected_index: i32,
  pub(super) mode: u32,
  /// The boxes round the annotations chosen together.
  pub(super) group: Vec<NativeAnnotationGroupBox>,
}

/// Settles what the native OSC is told about this layer's arrows, and what
/// the tool in hand means for the halo.
///
/// Answers the grips to publish and whether a halo was retired: changing the
/// tool in hand retires it, and so does holding no tool or the pen, because
/// the pointer may never move again to do it and nothing else clears it. So
/// does a list that no longer holds the haloed annotation where the halo is
/// placed - deleted, or moved by an undo - rather than let the halo pass to
/// whatever took its place. The next move halos again whatever is under it.
/// React is told because the keyboard acts on the arrow the halo is showing.
pub(super) fn apply_annotation_layout(
  manager: &mut super::state::PreviewManager,
  defaults: Option<AnnotationStyle>,
  counter_angle: Option<f64>,
  tool: Option<&str>,
  pane_index: Option<u32>,
  selected: &[String],
) -> (AnnotationLayout, bool) {
  let mode = annotation_mode(tool);
  manager.annotation_defaults = defaults;
  manager.annotation_counter_angle = counter_angle;
  let changed = manager.annotation_mode != mode;
  manager.annotation_mode = mode;
  manager.annotation_pane_index = pane_index;
  manager.annotation_selected = selected.to_vec();
  let retired = changed || hovers_nothing(mode) || manager.hover_lost();
  let hover_cleared = retired && manager.clear_annotation_hover();
  let layout = annotation_layout(manager, pane_index, mode, selected);
  (layout, hover_cleared)
}

/// The arrow grips for every layer: the chrome halos and picks an annotation
/// on any layer, so a choice never waits for its layer to be selected first.
/// The geometry comes from the manager's own output rather than the layout's
/// payload, so a gesture sample and the layout that echoes it publish the
/// same handles.
/// `selected` is every annotation chosen: one on its own shows its grips,
/// and several are marked and boxed as a group.
///
/// The selected pane's grips carry no layer of their own, as a still's
/// always have: the native chrome measures them against the selection, and
/// a press beside them lets the choice go. Every other layer's carry the
/// layer they are drawn on, which the chrome looks up among its targets.
pub(super) fn annotation_layout(
  manager: &super::state::PreviewManager,
  pane_index: Option<u32>,
  mode: u32,
  selected: &[String],
) -> AnnotationLayout {
  let mut layout = AnnotationLayout {
    handles: Vec::new(),
    paths: Vec::new(),
    selected_index: -1,
    mode,
    group: Vec::new(),
  };
  let grouped = |id: &str| selected.len() > 1 && selected.iter().any(|chosen| chosen == id);
  for (pane, annotations, source, image_width) in manager.published_panes() {
    let layer = if Some(pane) == pane_index {
      -1
    } else {
      pane as i32
    };
    if let [id] = selected {
      if let Some(index) = annotations.iter().position(|item| item.id == *id) {
        layout.selected_index = i32::try_from(layout.handles.len() + index).unwrap_or(-1);
      }
    }
    let handles = annotation_handles(annotations, source, image_width, &mut layout.paths);
    layout.handles.extend(
      handles
        .into_iter()
        .zip(annotations)
        .map(|(mut handle, annotation)| {
          handle.layer_id = layer;
          if grouped(&annotation.id) {
            handle.flags |= HANDLE_FLAG_GROUPED;
          }
          handle
        }),
    );
    // A still shows every annotation on its layer, so every member is shown.
    let members: Vec<_> = annotations
      .iter()
      .filter(|annotation| grouped(&annotation.id))
      .collect();
    layout.group.extend(group_boxes(
      &members,
      |_| true,
      source,
      crate::editor::annotations::snap::source_per_size(source, image_width),
      layer,
    ));
  }
  layout
}

impl super::state::PreviewManager {
  /// Every layer whose grips the chrome is given, in the order they are
  /// published: its list, its source's pixel size, and how wide its picture is
  /// drawn in points of annotation size.
  pub(super) fn published_panes(
    &self,
  ) -> impl Iterator<Item = (u32, &Vec<Annotation>, (u32, u32), f64)> + '_ {
    let count = self.output.as_ref().map_or(0, |output| output.items.len());
    (0..u32::try_from(count).unwrap_or(u32::MAX)).filter_map(move |pane| {
      Some((
        pane,
        self.annotations_for(pane)?,
        self.annotation_source(pane)?,
        self.annotation_image_width(pane)?,
      ))
    })
  }

  /// The annotation at `index` among the published grips, which is how the
  /// native chrome names one it halos or opens a menu on: its layer, its
  /// place in that layer's list, and the annotation.
  pub(super) fn published_annotation(&self, index: usize) -> Option<(u32, usize, &Annotation)> {
    let mut start = 0;
    for (pane, annotations, ..) in self.published_panes() {
      if let Some(annotation) = index.checked_sub(start).and_then(|at| annotations.get(at)) {
        return Some((pane, index - start, annotation));
      }
      start += annotations.len();
    }
    None
  }

  /// Renumbers the counter `id` that was just dropped on `pane_index`, which
  /// its own list numbered after that layer's counters alone, to follow every
  /// counter on the other layers too: a still's counters are one run across
  /// the whole picture, in the order they were placed.
  pub(super) fn number_fresh_counter(&mut self, pane_index: u32, id: &str) {
    let Some(output) = self.output.as_mut() else {
      return;
    };
    let elsewhere = output
      .items
      .iter()
      .enumerate()
      .filter(|(pane, _)| *pane != pane_index as usize)
      .flat_map(|(_, item)| &item.output.annotations)
      .filter(|annotation| matches!(annotation.shape, AnnotationShape::Counter { .. }))
      .count() as u32;
    let fresh = output.items.get_mut(pane_index as usize).and_then(|item| {
      item
        .output
        .annotations
        .iter_mut()
        .find(|annotation| annotation.id == id)
    });
    if let Some(Annotation {
      shape: AnnotationShape::Counter { value, .. },
      ..
    }) = fresh
    {
      *value += elsewhere;
    }
  }
}
