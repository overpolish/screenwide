// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tauri::command]
pub async fn start_recording_preview_player(
  app: AppHandle,
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  artifact_id: u64,
  settings: PreviewPlayerSettings,
  event_channel: Channel<RecordingPreviewPlayerEvent>,
  session_id: u64,
) -> Result<RecordingPreviewPlayerInfo, String> {
  let mut sources = sources(&app, artifact_id, Some(&settings))?;
  let info = RecordingPreviewPlayerInfo::from(&sources);
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if let Some(surface) = sources.preview_surface.as_ref() {
    if let Err(error) = surface.set_camera_effects(sources.camera_effects_now()) {
      eprintln!("Could not draw the camera's effects: {error}");
    }
  }
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  if session_id < manager.latest_session_id {
    return Ok(info);
  }
  manager.stop();
  manager.latest_session_id = session_id;
  // The callbacks name this session. They are installed only once it is the
  // session the manager holds, and under the same lock that adopts it: on
  // Windows one compositor belongs to the editor window and outlives every
  // session, so a superseded start installing its own would leave the live
  // session's native input reporting against clips nothing reads.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let annotation_clips = std::sync::Arc::clone(&sources.annotation_clips);
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if let Some(surface) = sources.preview_surface.as_mut() {
    let surface = Arc::get_mut(surface)
      .ok_or_else(|| "The recording preview surface is already in use".to_owned())?;
    super::super::annotation_bridge::install(surface, app.clone(), annotation_clips);
    {
      let pick_app = app.clone();
      surface.set_object_pick_callback(Box::new(move |press| {
        let app = pick_app.clone();
        // Picking reads and writes the project's picks, off the main thread.
        tauri::async_runtime::spawn_blocking(move || {
          let picked = crate::editor::camera_effects::picks::commands::add_pick(
            &app,
            artifact_id,
            &press.points,
            press.keep,
          );
          match picked {
            Ok(id) => super::super::choose_camera_object(&app, artifact_id, id),
            Err(error) => eprintln!("Could not pick the camera's object: {error}"),
          }
        });
      }));
    }
    let event_app = app.clone();
    surface.enable_editor(Box::new(move |zoom_percent| {
      let _ = event_app.emit(
        "recording-preview://transform",
        RecordingPreviewTransformEvent {
          session_id,
          zoom_percent,
        },
      );
    }));
    let event_app = app.clone();
    surface.set_selection_callback(Box::new(move |pane_index| {
      let _ = event_app.emit(
        "recording-preview://selection-change",
        RecordingPreviewSelectionChangeEvent {
          pane_index,
          session_id,
        },
      );
    }));
    if let Some(event_window) =
      app.get_webview_window(EditorKind::Recording.window_label().as_str())
    {
      let menu_window = event_window.clone();
      surface.set_pointer_down_callback(Box::new(move || {
        let _ = event_window.emit(
          crate::editor::preview_platform::NATIVE_POINTER_DOWN_EVENT,
          (),
        );
      }));
      let menu_app = app.clone();
      surface.set_context_menu_callback(Box::new(move |pane_index, annotation, x, y| {
        // A press on a word of the caption opens the captions' word menu.
        if pane_index == screenwide_workspace::CAPTIONS_LAYER_ID {
          let state = menu_app.state::<RecordingPreviewPlayerState>();
          let Ok(manager) = state.0.try_lock() else {
            return;
          };
          if let Some(menu) =
            annotation.and_then(|at| manager.caption_word_menu(at as usize, (x, y)))
          {
            let _ = menu_window.emit("preview://caption-context-menu", menu);
          }
          return;
        }
        // A press on an annotation opens its own menu, on either pane: the
        // menu offers what applies to it.
        let annotation_id = match annotation {
          None => None,
          Some(index) => {
            let state = menu_app.state::<RecordingPreviewPlayerState>();
            let Ok(manager) = state.0.try_lock() else {
              return;
            };
            match manager.annotation_targets().into_iter().nth(index as usize) {
              Some((_, annotation)) => Some(annotation.id),
              None => return,
            }
          }
        };
        let _ = menu_window.emit(
          crate::editor::preview_platform::NATIVE_CONTEXT_MENU_EVENT,
          RecordingPreviewContextMenuEvent {
            annotation_id,
            pane_index,
            x,
            y,
          },
        );
      }));
    }
    let event_app = app.clone();
    surface.set_selection_gesture_callback(Box::new(
      move |phase, pane_index, operation, edges, scale, delta_x, delta_y| {
        let manager = event_app.state::<RecordingPreviewPlayerState>();
        let composition = if let Ok(mut manager) = manager.0.try_lock() {
          let updated = manager
            .handle_selection_gesture(phase, pane_index, operation, edges, scale, delta_x, delta_y)
            .is_ok();
          // A Frame drag, or an Alt-drag Move growing the canvas around its
          // layer, moves the pane box on every pointer move, so the composition
          // follows it immediately: the paused still is redrawn from its
          // cached sources here, which also publishes the geometry the drag
          // deferred. Without this the pane would show the previous canvas
          // letterboxed into the new box until mouse-up.
          #[cfg(any(target_os = "macos", target_os = "windows"))]
          if updated
            && (operation == screenwide_workspace::editing::SelectionGestureOperation::FrameResize
              || (operation == screenwide_workspace::editing::SelectionGestureOperation::Move
                && edges & super::super::AUTO_FIT_MOVE_EDGE != 0))
            && !matches!(
              phase,
              screenwide_workspace::editing::SelectionGesturePhase::Begin
            )
          {
            // The surface state lock is not held here (`handle_editor_input`
            // emits its gestures after that scope ends), and nothing under
            // this call re-enters the manager, so the manager lock is only
            // ever taken before the surface lock - the same order every
            // command uses.
            let _ = manager.redraw_still_now();
          }
          #[cfg(not(target_os = "windows"))]
          let _ = updated;
          manager.selection_composition()
        } else {
          None
        };
        let phase = match phase {
          screenwide_workspace::editing::SelectionGesturePhase::Begin => "begin",
          screenwide_workspace::editing::SelectionGesturePhase::Update => "update",
          screenwide_workspace::editing::SelectionGesturePhase::End => "end",
          screenwide_workspace::editing::SelectionGesturePhase::Cancel => "cancel",
        };
        let operation = operation as u32;
        let _ = event_app.emit(
          "recording-preview://selection-gesture",
          RecordingPreviewSelectionGestureEvent {
            camera_overlay: composition.as_ref().map(|value| value.camera_overlay),
            delta_x,
            delta_y,
            edges,
            operation,
            pane_index,
            phase,
            recording_output: composition.map(|value| value.recording_output),
            scale,
            session_id,
          },
        );
      },
    ));
    surface.set_selection_snapping(true);
    surface.set_editor_active(false);
  }
  // A redaction drawn before its clip's first frame was decoded shows flat;
  // once the frame lands, a paused preview is drawn again with its fill.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  for held_fills in sources.held_fills.iter().flatten() {
    let ready_app = app.clone();
    held_fills.set_on_ready(Arc::new(move || {
      let state = ready_app.state::<RecordingPreviewPlayerState>();
      let Ok(mut manager) = state.0.lock() else {
        return;
      };
      if manager.session_id == Some(session_id) && !manager.is_playing {
        let _ = manager.restart(PlaybackMode::InteractiveStill);
      }
    }));
  }
  // A pinned annotation's path lands on a thread of its own: it is attached
  // to the clips as they stand and a paused preview is drawn again, and the
  // timeline hears how each path is coming along.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if let Some(pins) = sources.pins.as_ref() {
    let ready_app = app.clone();
    pins.set_on_ready(Arc::new(move || {
      let state = ready_app.state::<RecordingPreviewPlayerState>();
      let Ok(mut manager) = state.0.lock() else {
        return;
      };
      if manager.session_id != Some(session_id) {
        return;
      }
      if let Some(sources) = manager.sources.as_ref() {
        sources.reattach_pins(manager.position_ms);
      }
      if !manager.is_playing {
        // The path moves the annotation, so its grips follow it there.
        manager.publish_annotation_handles();
        let _ = manager.restart(PlaybackMode::InteractiveStill);
      }
    }));
    let status_app = app.clone();
    pins.set_on_status(Arc::new(move |status| {
      let _ = status_app.emit(
        "editor://recording-pin-status",
        screenwide_export::pin_paths::PinStatusEvent { session_id, status },
      );
    }));
  }
  sources.reattach_pins(0);
  manager.artifact_id = Some(artifact_id);
  manager.audio_indices = settings.audio.enabled_stream_indices;
  manager.audio_volumes = settings.audio.audio_track_volumes;
  manager.event_channel = Some(event_channel);
  manager.latest_layout_request = 0;
  manager.latest_seek_request = 0;
  manager.playback_rate = 1.0;
  manager.position_ms = 0;
  manager.sources = Some(sources);
  manager.session_id = Some(session_id);
  manager.restart(PlaybackMode::Still)?;
  Ok(info)
}
