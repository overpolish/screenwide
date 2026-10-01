// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The UI elements an arrow's tip can land on, detected from the frame under
//! the pointer.
//!
//! Detection is the ruler's own box detector over the whole frame, which costs
//! tens of milliseconds on a 5K screen: far too long to hold a pointer sample
//! for. The first sample that asks for a frame's elements therefore starts the
//! detection on a blocking thread and snaps to nothing; the first sample after
//! it lands snaps without needing a new press. One result is kept, because a
//! gesture only ever snaps to the frame it is drawn over.

use super::{element_padding, merge_fragments, SnapBounds};
use crate::ruler::analysis::{compute_gradients, detect_boxes, ComponentBox};
use crate::ruler::snapshot::Tolerance;
use std::sync::{Arc, Mutex};

/// Which frame a result belongs to: the artifact or session, the pane, and the
/// playhead the frame was decoded at. A still image has one frame per pane, so
/// its position is always zero.
pub(crate) type AnchorKey = (u64, u32, u64);

/// One frame's detected elements, as the padded rectangles an arrow's tip
/// rides, in source pixels.
///
/// Only the padded rectangles are kept, so growing thousands of rectangles per
/// pointer sample is never what makes the scan expensive. They are grown by
/// [`element_padding`] of the same source size and then run through
/// [`merge_fragments`], which folds the many small boxes of a dashed or dotted
/// glyph into the one element they make up.
pub(crate) struct AnchorBoxes {
  pub(crate) source_width: u32,
  pub(crate) source_height: u32,
  bounds: Vec<SnapBounds>,
}

impl AnchorBoxes {
  pub(crate) fn new(source_width: u32, source_height: u32, boxes: Vec<ComponentBox>) -> Self {
    let padding = element_padding((source_width, source_height));
    let padded = boxes
      .into_iter()
      .map(|item| SnapBounds::padded(item, padding))
      .collect();
    Self {
      source_width,
      source_height,
      bounds: merge_fragments(padded, (source_width, source_height)),
    }
  }

  /// Every element's padded rectangle, whose edges are the candidates.
  pub(crate) fn bounds(&self) -> &[SnapBounds] {
    &self.bounds
  }

  /// Whether these elements were detected at the size the annotations are
  /// measured in. A frame decoded at another size would place every edge
  /// somewhere else, so it is not offered at all rather than offered wrong.
  pub(crate) fn matches(&self, source: (u32, u32)) -> bool {
    self.source_width == source.0 && self.source_height == source.1
  }
}

/// Every element the detector finds in one RGBA8 frame.
///
/// Detected at the ruler's subtle tolerance: an arrow is aimed at whatever a
/// hand can see, including the low-contrast cards and dividers that the
/// balanced tolerance Recenter uses deliberately looks past.
pub(crate) fn detect_anchors(rgba: &[u8], width: u32, height: u32) -> AnchorBoxes {
  let expected = (width as usize)
    .checked_mul(height as usize)
    .and_then(|pixels| pixels.checked_mul(4));
  if !expected.is_some_and(|expected| expected > 0 && rgba.len() >= expected) {
    return AnchorBoxes::new(width, height, Vec::new());
  }
  let boxes = detect_boxes(
    &compute_gradients(rgba, width, height),
    Tolerance::SubtleEdges.threshold(),
  );
  AnchorBoxes::new(width, height, boxes)
}

#[derive(Default)]
struct CacheState {
  key: Option<AnchorKey>,
  anchors: Option<Arc<AnchorBoxes>>,
  detecting: Option<AnchorKey>,
}

/// The latest frame's elements. A new key replaces the one before it: the
/// editor only ever snaps to the frame the pointer is over, and a 5K frame's
/// anchor list is too large to keep a history of.
///
/// The mutex is a leaf. It is taken for a clone or a store and never while
/// waiting on anything else, so the pointer's thread can read it while holding
/// the manager and the detection thread can write it without holding one.
#[derive(Default)]
pub(crate) struct AnchorCache(Mutex<CacheState>);

impl AnchorCache {
  /// The elements detected for `key`, if they have landed.
  pub(crate) fn anchors(&self, key: AnchorKey) -> Option<Arc<AnchorBoxes>> {
    let state = self.0.lock().ok()?;
    (state.key == Some(key))
      .then(|| state.anchors.clone())
      .flatten()
  }

  pub(crate) fn clear(&self) {
    if let Ok(mut state) = self.0.lock() {
      *state = CacheState::default();
    }
  }

  /// Whether this caller is the one that runs detection for `key`: there is no
  /// result for it and none already on its way.
  fn claim(&self, key: AnchorKey) -> bool {
    let Ok(mut state) = self.0.lock() else {
      return false;
    };
    if state.detecting == Some(key) || (state.key == Some(key) && state.anchors.is_some()) {
      return false;
    }
    state.detecting = Some(key);
    true
  }

  fn store(&self, key: AnchorKey, anchors: AnchorBoxes) {
    if let Ok(mut state) = self.0.lock() {
      state.key = Some(key);
      state.anchors = Some(Arc::new(anchors));
      state.detecting = None;
    }
  }
}

/// Detects `key`'s elements off the pointer's thread unless its result is
/// already there or on its way. A frame `detect` cannot read reports no
/// elements, which is cached like any other answer so a frame that cannot be
/// decoded is not decoded again on the next sample.
pub(crate) fn request_anchors(
  cache: &Arc<AnchorCache>,
  key: AnchorKey,
  detect: impl FnOnce() -> AnchorBoxes + Send + 'static,
) {
  if !cache.claim(key) {
    return;
  }
  let cache = Arc::clone(cache);
  tauri::async_runtime::spawn_blocking(move || cache.store(key, detect()));
}
