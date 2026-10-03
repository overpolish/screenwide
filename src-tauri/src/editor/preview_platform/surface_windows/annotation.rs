// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotation tool's half of the Windows preview surface.
//!
//! The macOS surface hands this work to its Objective-C interaction view
//! (`recording_preview_surface_macos+annotation.m`); DirectComposition has no
//! such view, so the same picking and the same gesture state machine live
//! here. Both sides speak the layer's image-normalised space, never source
//! pixels, so everything above the platform facade stays identical.

use super::*;
/// What the pointer does over the picture, from the tool React has in hand,
/// and which shape a drawing mode makes. Both are declared once beside the
/// gesture model, so Windows and the macOS interaction view read the same
/// numbers.
use crate::editor::annotations::gesture::{drawing_kind, hovers_nothing, MODE_NONE};
use crate::editor::annotations::handles::{
  NativeAnnotationHandles, NativeAnnotationSnap, NativeGapSpan, SNAP_FLAG_ANCHOR, SNAP_FLAG_GAP_X,
  SNAP_FLAG_GAP_Y, SNAP_FLAG_GUIDE_X, SNAP_FLAG_GUIDE_Y,
};
use crate::editor::annotations::AnnotationKind;

/// The grip's hit box, matching the selection handles'.
const HANDLE_HIT: f64 = 8.0;

/// What a gesture acts on, matching `ScreenwideAnnotationTarget`.
const TARGET_NEW: u32 = 0;
const TARGET_EXISTING: u32 = 1;
const TARGET_NONE: u32 = 2;
const TARGET_SELECT: u32 = 3;
const TARGET_GROUP: u32 = 4;
const TARGET_TOGGLE: u32 = 5;
const TARGET_MARQUEE: u32 = 6;

/// Which grip a press took hold of, matching `ScreenwideAnnotationHandle`.
const HANDLE_MIDDLE: u32 = 1;
const HANDLE_END: u32 = 2;
const HANDLE_BODY: u32 = 3;
const HANDLE_TAIL: u32 = 4;

/// The published arrow chrome and the drag in progress over it.
pub(super) struct AnnotationState {
  /// Every arrow's grips, in the order the layers store them.
  pub(super) handles: Vec<NativeAnnotationHandles>,
  /// The strokes' fitted lines, normalised over their sources, which a
  /// stroke's grips point into.
  pub(super) paths: Vec<[f32; 2]>,
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
  /// The boxes round the annotations chosen together, as Rust published them.
  pub(super) group: Vec<crate::editor::annotations::group::NativeAnnotationGroupBox>,
  /// The marquee band being drawn, in display points.
  pub(super) marquee: Option<PreviewSurfaceRect>,
  drag: Option<Drag>,
  /// The box being typed into, if any.
  pub(super) typing: Option<typing::Typing>,
  /// Where the double-click that asked for a box to open landed, for the
  /// caret to start there once Rust opens it.
  opening: Option<(f64, f64)>,
  /// A press typing took - one that closed a box or opened one - whose drag
  /// and release belong to nothing else.
  press_taken: bool,
}

impl Default for AnnotationState {
  fn default() -> Self {
    Self {
      handles: Vec::new(),
      paths: Vec::new(),
      hover: None,
      hovered: -1,
      hover_started: None,
      hover_revision: 0,
      hover_refreshing: false,
      selected: -1,
      mode: MODE_NONE,
      snap: NativeAnnotationSnap::default(),
      group: Vec::new(),
      marquee: None,
      drag: None,
      typing: None,
      opening: None,
      press_taken: false,
    }
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
  /// arrow on empty picture (2). `paths` holds the strokes' fitted lines the
  /// grips point into.
  pub(crate) fn set_annotations(
    &self,
    handles: &[NativeAnnotationHandles],
    paths: &[[f32; 2]],
    selected_index: i32,
    mode: u32,
  ) {
    self.set_annotation_layer(handles, paths, selected_index, mode, -1);
  }

  pub(crate) fn set_annotation_layer(
    &self,
    handles: &[NativeAnnotationHandles],
    paths: &[[f32; 2]],
    selected_index: i32,
    mode: u32,
    _active_layer: i32,
  ) {
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    // A list that grew or shrank has moved what the hovered index names.
    let recounted = state.annotation.handles.len() != handles.len();
    state.annotation.handles.clear();
    state.annotation.handles.extend_from_slice(handles);
    state.annotation.paths.clear();
    state.annotation.paths.extend_from_slice(paths);
    state.annotation.selected = selected_index;
    let changed = state.annotation.mode != mode;
    state.annotation.mode = mode;
    // A new tool or list, no tool, or the pen: the halo goes, since the
    // pointer may never move again to retire it, and a pulse still running
    // must stop reporting it.
    if changed || recounted || hovers_nothing(mode) {
      state.annotation.hovered = -1;
      state.annotation.hover_revision += 1;
    }
    // The tool put down, or the box gone from under the typing: there is
    // nothing left to type into, and what was typed is kept.
    if typing::lost_its_box(&state) {
      self.inner.editor.finish_typing();
    }
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

/// A press the chrome has taken, before and after it becomes a drag.
mod drag;
use drag::Drag;

/// Resolving the picture an annotation is drawn in, and picking the grip or
/// shaft a press lands on.
mod picking;
/// The chrome's grips, whether it owns the screen, and its cursor.
mod picking_chrome;
/// How far a press is from each kind of annotation.
mod picking_distance;
use picking::{
  handle_at_point, image_frame, item_image_frame, layer_selection, selected_item, shaft_at_point,
  take_drawing_layer, text_geometry,
};
pub(super) use picking_chrome::{cursor_for, owns_chrome, selected_grips};

/// A redaction's box, grips and radius dot.
mod redact_chrome;
pub(super) use redact_chrome::selected_box as selected_redaction;

/// A magnifier's loupe: where it is picked and the grip that sets its size.
mod magnify_chrome;

/// What a snapped sample draws, and where.
mod snap_chrome;
pub(super) use snap_chrome::snap_chrome;

/// The boxes round a group, and the marquee band.
mod group_chrome;
pub(super) use group_chrome::{group_frames, has_group, marquee_frame};
use group_chrome::{group_layer_at_point, marquee_layer_at_point};

/// The halo that grows under the arrow the pointer rests on.
mod hover;
pub(crate) use hover::{refresh as refresh_hover, update as update_hover};

/// The gesture the press becomes, and the samples it reports.
mod gesture;
pub(super) use gesture::{cancel, choose_at, down, pointer_move, up};

/// Typing into a text box: the keyboard, the caret and the presses around it.
mod typing;
pub(super) use typing::{handle_typing_input, sync_typing_marks};
pub(crate) use typing::{open as open_text, press as typing_press};
pub(crate) use typing::{pointer_move as typing_move, up as typing_up};
