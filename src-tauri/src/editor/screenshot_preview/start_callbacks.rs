// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The native surface callbacks one screenshot preview session owns.
//!
//! Installed while the session is being adopted, never while its surface is
//! being obtained. On Windows one compositor belongs to the editor window and
//! outlives every session that draws on it, so callbacks installed by a start
//! that has already been superseded would otherwise replace the live
//! session's: the native tool would then report gestures against a session id
//! React has stopped listening for, and the arrow tool would silently draw
//! nothing for the rest of the process's life.

use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use super::super::preview_platform::{
  RecordingPreviewSurface, SelectionGestureOperation, SelectionGesturePhase,
};
use super::annotation_gesture::AnnotationCommit;
use super::payloads::{
  ScreenshotAnnotationChangeEvent, ScreenshotAnnotationHoverEvent, ScreenshotPreviewTransformEvent,
  ScreenshotSelectionChangeEvent, ScreenshotSelectionGestureEvent,
};
use super::state::ScreenshotPreviewState;
use crate::editor::annotations::gesture::AnnotationGestureTarget;

/// Hands a finished arrow gesture to React. Only the end of a gesture reports
/// one: everything in between is drawn natively from the manager's own
/// working copy, so the document takes exactly one edit per drag.
fn emit_annotation_change(app: &AppHandle, session_id: u64, commit: AnnotationCommit) {
  let _ = app.emit(
    "screenshot-preview://annotation-change",
    ScreenshotAnnotationChangeEvent {
      annotations: commit.annotations,
      pane_index: commit.pane_index,
      selected_annotation_id: commit.selected_annotation_id,
      session_id,
    },
  );
}

/// Tells React which annotation the halo is on. Only a change reports one: the
/// pulse runs at display rate, and the arrow under it is the same throughout.
pub(super) fn emit_annotation_hover(
  app: &AppHandle,
  session_id: u64,
  annotation_id: Option<String>,
) {
  let _ = app.emit(
    "screenshot-preview://annotation-hover",
    ScreenshotAnnotationHoverEvent {
      annotation_id,
      session_id,
    },
  );
}

pub(super) fn install(
  app: &AppHandle,
  window: &WebviewWindow,
  surface: &mut RecordingPreviewSurface,
  session_id: u64,
) {
  let event_app = app.clone();
  surface.enable_editor(Box::new(move |zoom_percent| {
    let _ = event_app.emit(
      "screenshot-preview://transform",
      ScreenshotPreviewTransformEvent {
        session_id,
        zoom_percent,
      },
    );
  }));
  surface.set_selection_snapping(true);
  let event_app = app.clone();
  surface.set_selection_callback(Box::new(move |pane_index| {
    let _ = event_app.emit(
      "screenshot-preview://selection-change",
      ScreenshotSelectionChangeEvent {
        pane_index,
        session_id,
      },
    );
  }));
  let event_window = window.clone();
  surface.set_pointer_down_callback(Box::new(move || {
    let _ = event_window.emit(
      super::super::preview_platform::NATIVE_POINTER_DOWN_EVENT,
      (),
    );
  }));
  let event_app = app.clone();
  surface.set_selection_gesture_callback(Box::new(
    move |phase, pane_index, operation, edges, scale, delta_x, delta_y| {
      let phase_name = match &phase {
        SelectionGesturePhase::Begin => "begin",
        SelectionGesturePhase::Update => "update",
        SelectionGesturePhase::End => "end",
        SelectionGesturePhase::Cancel => "cancel",
      };
      let manager = event_app.state::<ScreenshotPreviewState>();
      // Never wait for this mutex from AppKit's main thread. Surface
      // layout commands briefly mutate the manager on a worker and then
      // synchronously marshal geometry back to AppKit; blocking here
      // would invert those locks and freeze the entire application.
      match manager.0.try_lock() {
        Ok(mut manager) => {
          let _ = manager
            .handle_selection_gesture(phase, pane_index, operation, edges, scale, delta_x, delta_y);
        }
        Err(_) if matches!(phase, SelectionGesturePhase::End) => {
          let deferred_app = event_app.clone();
          tauri::async_runtime::spawn_blocking(move || {
            let state = deferred_app.state::<ScreenshotPreviewState>();
            let Ok(mut manager) = state.0.lock() else {
              return;
            };
            if manager.session_id == Some(session_id) {
              let _ = manager.handle_selection_gesture(
                phase, pane_index, operation, edges, scale, delta_x, delta_y,
              );
            }
          });
        }
        Err(_) => {}
      }
      let _ = event_app.emit(
        "screenshot-preview://selection-gesture",
        ScreenshotSelectionGestureEvent {
          delta_x,
          delta_y,
          edges,
          operation: match operation {
            SelectionGestureOperation::Move => 0,
            SelectionGestureOperation::Resize => 1,
            SelectionGestureOperation::Radius => 2,
            SelectionGestureOperation::FrameResize => 3,
            SelectionGestureOperation::FrameRadius => 4,
            SelectionGestureOperation::CropMove => 5,
            SelectionGestureOperation::CropResize => 6,
          },
          pane_index,
          phase: phase_name,
          scale,
          session_id,
        },
      );
    },
  ));
  let hover_app = app.clone();
  surface.set_annotation_hover_callback(Box::new(move |index, progress, image_points| {
    let state = hover_app.state::<ScreenshotPreviewState>();
    // Never wait on this mutex from AppKit's main thread: a halo frame is
    // the most droppable work there is, and the next one is sixteen
    // milliseconds away.
    let Ok(mut manager) = state.0.try_lock() else {
      return;
    };
    if manager.session_id != Some(session_id) {
      return;
    }
    let hovered = manager.handle_annotation_hover(index, progress, image_points);
    drop(manager);
    if let Some(annotation_id) = hovered {
      emit_annotation_hover(&hover_app, session_id, annotation_id);
    }
  }));
  let event_app = app.clone();
  surface.set_annotation_gesture_callback(Box::new(
    move |phase, pane_index, target_kind, index, handle, x, y, snap| {
      let Some(target) = AnnotationGestureTarget::from_raw(target_kind, index, handle) else {
        return;
      };
      let state = event_app.state::<ScreenshotPreviewState>();
      // Never wait for this mutex from AppKit's main thread: see the
      // selection gesture above for why that inverts the locks.
      match state.0.try_lock() {
        Ok(mut manager) => {
          if let Some(commit) =
            manager.handle_annotation_gesture(phase, pane_index, target, x, y, snap)
          {
            emit_annotation_change(&event_app, session_id, commit);
          }
        }
        Err(_) if matches!(phase, SelectionGesturePhase::End) => {
          // The commit is the whole point of a mouse-up, so a contended
          // lock defers it rather than dropping it.
          let deferred_app = event_app.clone();
          tauri::async_runtime::spawn_blocking(move || {
            let state = deferred_app.state::<ScreenshotPreviewState>();
            let Ok(mut manager) = state.0.lock() else {
              return;
            };
            if manager.session_id != Some(session_id) {
              return;
            }
            if let Some(commit) =
              manager.handle_annotation_gesture(phase, pane_index, target, x, y, snap)
            {
              emit_annotation_change(&deferred_app, session_id, commit);
            }
          });
        }
        Err(_) => {}
      };
    },
  ));
}
