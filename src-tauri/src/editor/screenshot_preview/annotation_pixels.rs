// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the annotation tools read of a pane's picture: the elements an
//! arrow's tip snaps to and the pixels a highlight selects from, and the snap
//! chrome that shows what a sample landed on.

use super::state::PreviewManager;
use crate::editor::annotations::handles::annotation_snap;
use crate::editor::annotations::highlight::picture::HighlightPicture;
use crate::editor::annotations::snap::{detect_anchors, request_anchors, AnchorBoxes, SnapResult};
use std::sync::Arc;

impl PreviewManager {
  /// The elements detected in this pane's image, for an arrow's tip to land
  /// on. The first call starts the detection and answers `None`; the pointer
  /// is never held for it, and every later sample reads the cached result.
  pub(super) fn annotation_anchors(
    &self,
    pane_index: u32,
    source: (u32, u32),
  ) -> Option<Arc<AnchorBoxes>> {
    let key = (self.session_id?, pane_index, 0);
    if let Some(anchors) = self.annotation_anchor_cache.anchors(key) {
      return anchors.matches(source).then_some(anchors);
    }
    let item_id = self.output.as_ref()?.items.get(pane_index as usize)?.id;
    let image = self
      .sources
      .iter()
      .find(|source| source.id == item_id)
      .map(|source| Arc::clone(&source.image))?;
    request_anchors(&self.annotation_anchor_cache, key, move || {
      detect_anchors(&image.rgba, image.width, image.height)
    });
    None
  }

  /// Publishes what the sample on screen snapped to. It goes out before the
  /// grips, because on Windows publishing those is what redraws the chrome.
  pub(super) fn publish_annotation_snap(&self, pane_index: u32, result: &SnapResult) {
    if let (Some(surface), Some(source)) =
      (self.surface.as_ref(), self.annotation_source(pane_index))
    {
      surface.set_annotation_snap_guides(annotation_snap(result, source));
    }
  }

  /// The pane's picture, for a highlight to select from. A screenshot's is
  /// always at hand, at the source's own size.
  pub(super) fn annotation_picture(&self, pane_index: u32) -> Option<Arc<HighlightPicture>> {
    let item_id = self.output.as_ref()?.items.get(pane_index as usize)?.id;
    let source = self.sources.iter().find(|source| source.id == item_id)?;
    let size = (source.image.width, source.image.height);
    HighlightPicture::new(Arc::clone(&source.image), size).map(Arc::new)
  }
}
