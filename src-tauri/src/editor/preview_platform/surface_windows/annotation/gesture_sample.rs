// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A press's pointer samples, resolved against the published chrome under the
//! surface lock and reported to the manager without it.

use super::*;

/// One resolved pointer sample, in the layer's image-normalised space and
/// addressed the way everything above the facade expects: by layer and by the
/// arrow's index within it. Resolved under the state lock, reported without.
#[derive(Clone, Copy)]
pub(super) struct Sample {
  phase: SelectionGesturePhase,
  layer: u32,
  target_kind: u32,
  index: u32,
  handle: u32,
  x: f64,
  y: f64,
  /// Which snapping modifiers were held when the sample was taken: bit 0
  /// Shift, which holds a counter's tail to the quarter turns, and bit 1
  /// Ctrl, which snaps the position itself.
  snap: u32,
  /// How wide the layer's picture is drawn on screen, in display points,
  /// which is what turns a snap's reach into source pixels.
  image_points: f64,
}

/// Which snapping modifiers are down. Read at the moment a sample is resolved
/// rather than latched at the press, so either can be taken and let go part
/// way through a drag, exactly as the macOS view reads
/// `NSEvent.modifierFlags`. Ctrl here is Command there.
fn snapped() -> u32 {
  use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_SHIFT};
  let down = |key: u16| unsafe { GetKeyState(i32::from(key)) < 0 };
  u32::from(down(VK_SHIFT.0)) | (u32::from(down(VK_CONTROL.0)) << 1)
}

/// Whether a press takes the toggle modifier: Ctrl, as the timeline's own
/// lanes add an item to their choice. Command on macOS.
pub(super) fn press_toggles() -> bool {
  snapped() & 2 != 0
}

/// Resolves a sample against the published chrome. `None` when the point has
/// no picture to be normalised against, which is nothing to report.
pub(super) fn resolve(
  state: &SurfaceState,
  phase: SelectionGesturePhase,
  target_kind: u32,
  index: u32,
  handle: u32,
  point: (f64, f64),
) -> Option<Sample> {
  // Windows draws every layer into its own pane, but the gesture addresses
  // the layer, which is the identity a selection gesture reports too.
  let mut layer = state.selection.map_or(0, |selection| selection.layer_id);
  let mut index = index;
  let mut image = image_frame(state);
  if matches!(target_kind, TARGET_EXISTING | TARGET_SELECT | TARGET_TOGGLE) {
    if let Some(item) = state.annotation.handles.get(index as usize) {
      // An annotation is measured on its own layer, whichever one is chosen.
      image = item_image_frame(state, index as i32);
      if item.layer_id >= 0 {
        layer = item.layer_id as u32;
      }
      index = item.index;
    }
  }
  let image = image?;
  Some(Sample {
    phase,
    layer,
    target_kind,
    index,
    handle,
    x: (point.0 - image.x) / image.width,
    y: (point.1 - image.y) / image.height,
    snap: snapped(),
    image_points: image.width,
  })
}

/// Resolves a sample of a gesture measured on `layer` rather than on one
/// annotation: the chosen group carried, or a marquee band's corners.
pub(super) fn resolve_layer(
  state: &SurfaceState,
  phase: SelectionGesturePhase,
  target_kind: u32,
  layer: i32,
  point: (f64, f64),
) -> Option<Sample> {
  let image = super::super::picking::layer_image_rect(state, layer)?;
  // A still's annotations carry no layer of their own: the selection's is
  // theirs, as it is for every other gesture on a still.
  let layer = u32::try_from(layer)
    .ok()
    .or_else(|| state.selection.map(|selection| selection.layer_id))?;
  Some(Sample {
    phase,
    layer,
    target_kind,
    index: 0,
    handle: HANDLE_BODY,
    x: (point.0 - image.x) / image.width,
    y: (point.1 - image.y) / image.height,
    snap: snapped(),
    image_points: image.width,
  })
}

/// Reports resolved samples. MUST be called with no surface state held: the
/// callback re-enters the surface to publish what it changed.
pub(super) fn report(inner: &SurfaceInner, samples: &[Sample]) {
  if samples.is_empty() {
    return;
  }
  let Ok(mut callbacks) = inner.callbacks.lock() else {
    return;
  };
  let Some(callback) = callbacks.annotation_gesture.as_mut() else {
    return;
  };
  for sample in samples {
    callback(
      sample.phase,
      sample.layer,
      sample.target_kind,
      sample.index,
      sample.handle,
      sample.x,
      sample.y,
      sample.snap,
      sample.image_points,
    );
  }
}
