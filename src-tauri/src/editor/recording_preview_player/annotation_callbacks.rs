// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(crate) fn install(
  surface: &mut RecordingPreviewSurface,
  app: AppHandle,
  clips: Arc<RwLock<Vec<RecordingAnnotationClip>>>,
) {
  let event_clips = Arc::clone(&clips);
  let event_app = app.clone();
  surface.set_annotation_gesture_callback(Box::new(
    move |phase, pane, kind, index, handle, x, y| {
      let Some(target) = AnnotationGestureTarget::from_raw(kind, index, handle) else {
        return;
      };
      let state = event_app.state::<RecordingPreviewPlayerState>();
      let attempt = state.0.try_lock();
      match attempt {
        Ok(mut manager) => {
          if !manager
            .sources
            .as_ref()
            .is_some_and(|sources| Arc::ptr_eq(&sources.annotation_clips, &event_clips))
          {
            return;
          }
          if let Some(commit) = manager.annotation_gesture(phase, pane, target, x, y) {
            let _ = event_app.emit("editor://recording-annotations", commit);
          }
        }
        Err(_)
          if matches!(
            phase,
            SelectionGesturePhase::End | SelectionGesturePhase::Cancel
          ) =>
        {
          let deferred = event_app.clone();
          let deferred_clips = Arc::clone(&event_clips);
          tauri::async_runtime::spawn_blocking(move || {
            if let Ok(mut manager) = deferred.state::<RecordingPreviewPlayerState>().0.lock() {
              if !manager
                .sources
                .as_ref()
                .is_some_and(|sources| Arc::ptr_eq(&sources.annotation_clips, &deferred_clips))
              {
                return;
              }
              if let Some(commit) = manager.annotation_gesture(phase, pane, target, x, y) {
                let _ = deferred.emit("editor://recording-annotations", commit);
              }
            }
          });
        }
        _ => {}
      }
    },
  ));
  surface.set_annotation_hover_callback(Box::new(move |index, _, _| {
    let state = app.state::<RecordingPreviewPlayerState>();
    let Ok(manager) = state.0.try_lock() else {
      return;
    };
    if !manager
      .sources
      .as_ref()
      .is_some_and(|sources| Arc::ptr_eq(&sources.annotation_clips, &clips))
    {
      return;
    }
    let Some(session_id) = manager.session_id else {
      return;
    };
    let annotation_id = manager
      .annotation_targets()
      .get(index as usize)
      .map(|(_, mark)| mark.id.clone());
    let _ = app.emit(
      "editor://recording-annotation-hover",
      Hover {
        session_id,
        annotation_id,
      },
    );
  }));
}
