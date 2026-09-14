// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use super::super::preview_platform::{
  RecordingPreviewSurface, SelectionGestureOperation, SelectionGesturePhase,
};
use super::super::{EditorArtifact, EditorKind, EditorState};
#[cfg(target_os = "macos")]
use super::annotation_gesture::AnnotationCommit;
#[cfg(target_os = "macos")]
use super::annotation_target::AnnotationGestureTarget;
#[cfg(target_os = "macos")]
use super::payloads::ScreenshotAnnotationChangeEvent;
use super::payloads::{
  ScreenshotPreviewTransformEvent, ScreenshotSelectionChangeEvent, ScreenshotSelectionGestureEvent,
};
use super::state::ScreenshotPreviewState;

/// Hands a finished arrow gesture to React. Only the end of a gesture reports
/// one: everything in between is drawn natively from the manager's own
/// working copy, so the document takes exactly one edit per drag.
#[cfg(target_os = "macos")]
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

#[tauri::command]
pub fn start_screenshot_preview(
  app: AppHandle,
  state: tauri::State<'_, ScreenshotPreviewState>,
  artifact_id: u64,
  session_id: u64,
) -> Result<(), String> {
  let sources = {
    let editor_state = app.state::<EditorState>();
    let artifact = editor_state
      .screenshot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(EditorArtifact::Screenshot { id, items, .. }) = artifact.as_ref() else {
      return Err("There is no screenshot to preview".to_owned());
    };
    if *id != artifact_id {
      return Err("That screenshot is no longer available in the editor".to_owned());
    }
    items
      .iter()
      .map(|item| (item.id, Arc::new(item.image.clone())))
      .collect::<Vec<_>>()
  };
  let surface = app
    .get_webview_window(EditorKind::Screenshot.window_label().as_str())
    .map(|window| {
      let mut surface = RecordingPreviewSurface::from_window(&window)?;
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      {
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
                let _ = manager.handle_selection_gesture(
                  phase, pane_index, operation, edges, scale, delta_x, delta_y,
                );
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
        #[cfg(target_os = "macos")]
        {
          let event_app = app.clone();
          let hover_app = app.clone();
          surface.set_annotation_hover_callback(Box::new(
            move |index, progress, image_points| {
              let state = hover_app.state::<ScreenshotPreviewState>();
              // Never wait on this mutex from AppKit's main thread: a halo
              // frame is the most droppable work there is, and the next one
              // is sixteen milliseconds away.
              let Ok(mut manager) = state.0.try_lock() else {
                return;
              };
              if manager.session_id == Some(session_id) {
                manager.handle_annotation_hover(index, progress, image_points);
              }
            },
          ));
          surface.set_annotation_gesture_callback(Box::new(
            move |phase, pane_index, target_kind, index, handle, x, y| {
              let Some(target) = AnnotationGestureTarget::from_raw(target_kind, index, handle)
              else {
                return;
              };
              let state = event_app.state::<ScreenshotPreviewState>();
              // Never wait for this mutex from AppKit's main thread: see the
              // selection gesture above for why that inverts the locks.
              match state.0.try_lock() {
                Ok(mut manager) => {
                  if let Some(commit) =
                    manager.handle_annotation_gesture(phase, pane_index, target, x, y)
                  {
                    emit_annotation_change(&event_app, session_id, commit);
                  }
                }
                Err(_) if matches!(phase, SelectionGesturePhase::End) => {
                  // The commit is the whole point of a mouse-up, so a
                  // contended lock defers it rather than dropping it.
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
                      manager.handle_annotation_gesture(phase, pane_index, target, x, y)
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
      }
      Ok::<_, String>(Arc::new(surface))
    })
    .transpose()?;
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The screenshot preview is unavailable".to_owned())?;
  if session_id < manager.latest_session_id {
    return Ok(());
  }
  manager.stop();
  manager.latest_session_id = session_id;
  manager.session_id = Some(session_id);
  manager.sources = sources;
  manager.surface = surface;
  Ok(())
}
