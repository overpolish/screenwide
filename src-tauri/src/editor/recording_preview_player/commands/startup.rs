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
      surface.set_context_menu_callback(Box::new(move |pane_index, x, y| {
        let _ = menu_window.emit(
          crate::editor::preview_platform::NATIVE_CONTEXT_MENU_EVENT,
          RecordingPreviewContextMenuEvent { pane_index, x, y },
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
          // follows it immediately. macOS recomposes natively; Windows redraws the
          // paused still from its cached sources here, which also publishes
          // the geometry the drag deferred. Without this the pane would show
          // the previous canvas letterboxed into the new box until mouse-up.
          #[cfg(target_os = "windows")]
          if updated
            && (operation
              == crate::editor::preview_platform::SelectionGestureOperation::FrameResize
              || (operation == crate::editor::preview_platform::SelectionGestureOperation::Move
                && edges & super::super::AUTO_FIT_MOVE_EDGE != 0))
            && !matches!(
              phase,
              crate::editor::preview_platform::SelectionGesturePhase::Begin
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
          crate::editor::preview_platform::SelectionGesturePhase::Begin => "begin",
          crate::editor::preview_platform::SelectionGesturePhase::Update => "update",
          crate::editor::preview_platform::SelectionGesturePhase::End => "end",
          crate::editor::preview_platform::SelectionGesturePhase::Cancel => "cancel",
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
