// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotation tool's half of the Windows preview surface.
//!
//! The Metal backend hands this work to its Objective-C interaction view
//! (`recording_preview_surface_macos+annotation.m`); DirectComposition has no
//! such view, so the same picking and the same gesture state machine live
//! here. Both sides speak the layer's image-normalised space, never source
//! pixels, so everything above the platform facade stays identical.

use super::*;
use crate::editor::annotations::handles::{
  NativeAnnotationHandles, NativeAnnotationSnap, NativeGapSpan, SNAP_FLAG_ANCHOR, SNAP_FLAG_GAP_X,
  SNAP_FLAG_GAP_Y, SNAP_FLAG_GUIDE_X, SNAP_FLAG_GUIDE_Y,
};

/// A press has to travel this far before it draws an arrow rather than
/// clearing the choice: a click and a very short drag are the same gesture to
/// a hand, and neither should leave a stub behind.
const DRAG_SLOP: f64 = 3.0;
/// The grip's hit box, matching the selection handles'.
const HANDLE_HIT: f64 = 8.0;

/// What the pointer does over the picture, from the tool React has in hand.
const MODE_NONE: u32 = 0;
const MODE_ARROW: u32 = 2;
const MODE_COUNTER: u32 = 3;

/// What a gesture acts on, matching `ScreenwideAnnotationTarget`.
const TARGET_NEW: u32 = 0;
const TARGET_EXISTING: u32 = 1;
const TARGET_NONE: u32 = 2;
const TARGET_SELECT: u32 = 3;

/// Which grip a press took hold of, matching `ScreenwideAnnotationHandle`.
const HANDLE_END: u32 = 2;
const HANDLE_BODY: u32 = 3;
const HANDLE_TAIL: u32 = 4;

/// The published arrow chrome and the drag in progress over it.
pub(super) struct AnnotationState {
  /// Every arrow's grips, in the order the layers store them.
  pub(super) handles: Vec<NativeAnnotationHandles>,
  /// The arrow the pointer rests on: its layer, its place in that layer's
  /// list, and how wide its halo has grown, in canvas pixels. Preview chrome
  /// only - the export never sees it.
  pub(super) hover: Option<(u64, usize, f32)>,
  /// Which arrow the pulse is on, and when it started, so every frame of the
  /// halo is derived from the clock rather than integrated.
  pub(super) hovered: i32,
  pub(super) hover_started: Option<std::time::Instant>,
  /// Bumped whenever the hovered arrow changes, so a pulse that is no longer
  /// the current one stops reporting.
  pub(super) hover_revision: u64,
  /// A re-measure of the halo against the picture's current size is already
  /// on its way, so a zoom drag that moves the transform every sample asks
  /// for one report rather than one per sample.
  pub(super) hover_refreshing: bool,
  /// The arrow whose grips are drawn and hit-tested, or -1 for none.
  pub(super) selected: i32,
  pub(super) mode: u32,
  /// What the last gesture sample snapped to, for the chrome to draw. Zeroed
  /// whenever a sample snaps to nothing, and when the gesture ends.
  pub(super) snap: NativeAnnotationSnap,
  drag: Option<Drag>,
}

impl Default for AnnotationState {
  fn default() -> Self {
    Self {
      handles: Vec::new(),
      hover: None,
      hovered: -1,
      hover_started: None,
      hover_revision: 0,
      hover_refreshing: false,
      selected: -1,
      mode: MODE_NONE,
      snap: NativeAnnotationSnap::default(),
      drag: None,
    }
  }
}

/// A press the arrow chrome has taken, before and after it becomes a drag.
#[derive(Clone, Copy)]
struct Drag {
  target_kind: u32,
  index: u32,
  handle: u32,
  origin: (f64, f64),
  /// The press has not travelled far enough to be a drag yet, so no gesture
  /// has begun and nothing has been edited.
  pending: bool,
  begun: bool,
}

impl Drag {
  /// A press the chrome has taken but which has not travelled yet.
  fn pending(target_kind: u32, index: u32, handle: u32, origin: (f64, f64)) -> Self {
    Self {
      target_kind,
      index,
      handle,
      origin,
      pending: true,
      begun: false,
    }
  }

  /// A press that is a gesture from the moment it lands: the counter tool drops
  /// an annotation where it is pressed rather than drawing one out, so a click
  /// alone commits it.
  fn begun(target_kind: u32, index: u32, handle: u32, origin: (f64, f64)) -> Self {
    Self {
      target_kind,
      index,
      handle,
      origin,
      pending: false,
      begun: true,
    }
  }

  /// Advances the drag by one pointer sample, reporting whether this sample
  /// is the one that turns the press into a gesture. A press that has not
  /// travelled past the slop yet leaves nothing behind: no phase is emitted
  /// and no edit is made, so a click never nudges the arrow.
  fn sample(&mut self, point: (f64, f64)) -> bool {
    if !self.pending {
      return false;
    }
    if (point.0 - self.origin.0).hypot(point.1 - self.origin.1) < DRAG_SLOP {
      return false;
    }
    self.pending = false;
    self.begun = true;
    true
  }

  /// Whether this sample should report movement at all. A press still inside
  /// the slop reports nothing.
  fn reports(&self) -> bool {
    !self.pending
  }
}

impl RecordingPreviewSurface {
  /// Publishes the halo the preview draws: the hovered arrow's layer, its
  /// place in that layer's list, and the halo's width in canvas pixels. The
  /// export never sees it, which is why it rides here rather than on the
  /// document.
  pub(crate) fn set_annotation_hover(&self, hover: Option<(u64, usize, f32)>) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.annotation.hover = hover;
    }
  }

  /// Moves the halo and redraws every pane from what it last composed. For a
  /// recording, whose frames the player owns, this is the only way a halo
  /// frame reaches the screen without asking the decoder for the frame again.
  pub(crate) fn redraw_annotation_hover(&self, hover: Option<(u64, usize, f32)>) {
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    if state.annotation.hover == hover {
      return;
    }
    state.annotation.hover = hover;
    redraw_composed_panes(&self.inner, &mut state);
  }

  /// Publishes what the sample on screen snapped to. It is only stored: a
  /// sample's annotations arrive immediately after and redraw the held frame
  /// together with this chrome, so drawing here would draw the overlay twice
  /// and leave the guides a frame ahead of the geometry they explain.
  pub(crate) fn set_annotation_snap_guides(&self, snap: NativeAnnotationSnap) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.annotation.snap = snap;
    }
  }
  /// Publishes the selected layer's arrow grips. `selected_index` is the
  /// arrow whose three handles are drawn, or -1 for none; `mode` is what the
  /// pointer does over the picture: nothing (0), hit-test the arrows that are
  /// there and otherwise fall through to the layer (1), or also draw a new
  /// arrow on empty picture (2).
  pub(crate) fn set_annotations(
    &self,
    handles: &[NativeAnnotationHandles],
    selected_index: i32,
    mode: u32,
  ) {
    self.set_annotation_layer(handles, selected_index, mode, -1);
  }

  pub(crate) fn set_annotation_layer(
    &self,
    handles: &[NativeAnnotationHandles],
    selected_index: i32,
    mode: u32,
    _active_layer: i32,
  ) {
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    state.annotation.handles.clear();
    state.annotation.handles.extend_from_slice(handles);
    state.annotation.selected = selected_index;
    state.annotation.mode = mode;
    // The grips are chrome: publishing them is what moves them on screen,
    // and what stands the layer's own chrome up or down. The cursor needs no
    // push - the window re-asks on the next pointer move.
    draw_selection(&self.inner, &state);
  }

  pub(crate) fn set_annotation_gesture_callback(&mut self, callback: AnnotationGestureCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.annotation_gesture = Some(callback);
    }
  }

  pub(crate) fn set_annotation_hover_callback(&mut self, callback: AnnotationHoverCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.annotation_hover = Some(callback);
    }
  }
}

/// Turning a layer's annotations into what the compositor draws this frame.
#[path = "annotation/prepare.rs"]
mod prepare;
pub(crate) use prepare::{placed_arrows, prepared_arrows};

/// Resolving the picture an annotation is drawn in, and picking the grip or
/// shaft a press lands on.
#[path = "annotation/picking.rs"]
mod picking;
pub(super) use picking::{cursor_for, owns_chrome, selected_grips};
use picking::{
  handle_at_point, image_extent, image_frame, item_image_frame, layer_selection, normalised_point,
  selected_item, shaft_at_point,
};

/// What a snapped sample draws, and where.
#[path = "annotation/snap_chrome.rs"]
mod snap_chrome;
pub(super) use snap_chrome::{snap_chrome, SnapChrome};

/// The halo that grows under the arrow the pointer rests on.
#[path = "annotation/hover.rs"]
mod hover;
pub(crate) use hover::{refresh as refresh_hover, update as update_hover};

/// The gesture the press becomes, and the samples it reports.
#[path = "annotation/gesture.rs"]
mod gesture;
pub(super) use gesture::{cancel, down, pointer_move, up};

#[cfg(test)]
#[path = "annotation/drag_tests.rs"]
mod drag_tests;
