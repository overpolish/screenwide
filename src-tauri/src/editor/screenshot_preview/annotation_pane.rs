// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the annotation gestures read of a pane: its source's size, how wide
//! its picture is drawn, and the annotations on it.

use super::state::PreviewManager;
use screenwide_annotations::Annotation;

impl PreviewManager {
  /// The source's pixel dimensions for one pane, for the normalised points
  /// the native side reports.
  pub(super) fn annotation_source(&self, pane_index: u32) -> Option<(u32, u32)> {
    let output = self.output.as_ref()?;
    let item = output.items.get(pane_index as usize)?;
    let source = self.sources.iter().find(|source| source.id == item.id)?;
    Some((source.image.width, source.image.height))
  }

  /// How wide this pane's picture is drawn, in points of annotation size: what
  /// turns a size into source pixels, or into a share of the picture.
  pub(super) fn annotation_image_width(&self, pane_index: u32) -> Option<f64> {
    let item = self.output.as_ref()?.items.get(pane_index as usize)?;
    let source = self.sources.iter().find(|source| source.id == item.id)?;
    Some(item.output.image_width / source.capture.size_scale())
  }

  pub(super) fn annotations_for(&self, pane_index: u32) -> Option<&Vec<Annotation>> {
    Some(
      &self
        .output
        .as_ref()?
        .items
        .get(pane_index as usize)?
        .output
        .annotations,
    )
  }

  pub(super) fn pane_annotations_mut(&mut self, pane_index: u32) -> Option<&mut Vec<Annotation>> {
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
}
