// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Screenshot-specific annotation layout and hover state. Shared handles and
//! editing live in the editor's annotation module.

use crate::editor::annotations::handles::{annotation_handles, NativeAnnotationHandles};
use crate::editor::annotations::AnnotationStyle;

/// What the pointer does over the picture while a tool is in hand. The select
/// tool hit-tests the arrows already there and lets every other press fall
/// through to the layer; the arrow tool also draws a new one on empty
/// picture. The values are the native `ScreenwideAnnotationMode`.
pub(super) const ANNOTATION_MODE_NONE: u32 = 0;
pub(super) const ANNOTATION_MODE_SELECT: u32 = 1;
pub(super) const ANNOTATION_MODE_ARROW: u32 = 2;

/// The tool name React sends, as a mode. Anything else puts the chrome away.
pub(super) fn annotation_mode(tool: Option<&str>) -> u32 {
  match tool {
    Some("arrow") => ANNOTATION_MODE_ARROW,
    Some("select") => ANNOTATION_MODE_SELECT,
    _ => ANNOTATION_MODE_NONE,
  }
}

/// Where the hover halo is, and how wide. The width is already in the layer's
/// canvas pixels: the native side reports how big the picture is drawn, and
/// the manager knows how many canvas pixels that is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AnnotationHover {
  pub(crate) index: usize,
  pub(crate) layer_id: u64,
  pub(crate) width: f32,
}

/// The halo's width in display points over the pulse, eased the way the
/// ruler's is: three points growing to eight over 160 ms.
pub(super) fn hover_width_points(progress: f64) -> f64 {
  let eased = 1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3);
  3.0 + (8.0 - 3.0) * eased
}

/// What one layout push tells the native OSC about this layer's arrows.
pub(super) struct AnnotationLayout {
  pub(super) handles: Vec<NativeAnnotationHandles>,
  /// The arrow whose three grips are drawn, or -1 for none.
  pub(super) selected_index: i32,
  pub(super) mode: u32,
}

/// Settles what the native OSC is told about this layer's arrows, and what
/// the tool in hand means for the halo.
///
/// Answers the grips to publish and whether a halo was retired: putting the
/// tool down retires it, because the pointer may never move again to do it
/// and nothing else clears it, and React is told because the keyboard acts on
/// the arrow the halo is showing.
pub(super) fn apply_annotation_layout(
  manager: &mut super::state::PreviewManager,
  defaults: Option<AnnotationStyle>,
  tool: Option<&str>,
  pane_index: Option<u32>,
  selected: Option<&str>,
) -> (AnnotationLayout, bool) {
  let mode = annotation_mode(tool);
  manager.annotation_defaults = defaults;
  manager.annotation_mode = mode;
  manager.annotation_pane_index = pane_index;
  let hover_cleared = mode == ANNOTATION_MODE_NONE && manager.clear_annotation_hover();
  let layout = annotation_layout(manager, pane_index, mode, selected);
  (layout, hover_cleared)
}

/// The arrow grips for the pane the OSC is drawn against. The geometry comes
/// from the manager's own output rather than the layout's payload, so a
/// gesture sample and the layout that echoes it publish the same handles.
fn annotation_layout(
  manager: &super::state::PreviewManager,
  pane_index: Option<u32>,
  mode: u32,
  selected: Option<&str>,
) -> AnnotationLayout {
  let handles = pane_index
    .and_then(|pane_index| {
      let source = manager.annotation_source(pane_index)?;
      Some(annotation_handles(
        manager.annotations_for(pane_index)?,
        source,
        manager.annotation_image_width(pane_index)?,
      ))
    })
    .unwrap_or_default();
  let selected_index = pane_index
    .and_then(|pane_index| manager.annotations_for(pane_index))
    .zip(selected)
    .and_then(|(annotations, id)| annotations.iter().position(|item| item.id == id))
    .map_or(-1, |index| i32::try_from(index).unwrap_or(-1));
  AnnotationLayout {
    handles,
    selected_index,
    mode,
  }
}
