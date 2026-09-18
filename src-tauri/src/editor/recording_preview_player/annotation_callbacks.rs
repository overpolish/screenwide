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
    move |phase, pane, kind, index, handle, x, y, snap| {
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
          if let Some(commit) = manager.annotation_gesture(phase, pane, target, x, y, snap) {
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
              if let Some(commit) = manager.annotation_gesture(phase, pane, target, x, y, snap) {
                let _ = deferred.emit("editor://recording-annotations", commit);
              }
            }
          });
        }
        _ => {}
      }
    },
  ));
  surface.set_annotation_hover_callback(Box::new(move |index, progress, image_points| {
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
    // Both backends redraw the halo without recomposing: the D3D11 panes
    // redraw from what they last composed, and the retained Metal scene has
    // the halo set on the annotations it is already holding.
    if let Some(surface) = manager
      .sources
      .as_ref()
      .and_then(|sources| sources.preview_surface.as_ref())
    {
      // The annotation's own pane, not the selected one: a shortcut or the
      // camera can hold the selection while the pointer rests on an arrow over
      // the screen. `annotation_targets` reports each annotation beside the
      // pane it was drawn over, in the order the grips were published.
      let halo = usize::try_from(index)
        .ok()
        .and_then(|index| {
          let targets = manager.annotation_targets();
          let (pane, _) = targets.get(index)?;
          // The compositor places a halo by the annotation's index within its
          // own pane's list, so the flat published index has to be counted down
          // to a local one.
          let local = targets[..index]
            .iter()
            .filter(|(candidate, _)| candidate == pane)
            .count();
          Some((*pane, local))
        })
        .and_then(|(pane, local)| {
          if !image_points.is_finite() || image_points <= 0.0 {
            return None;
          }
          let output_width = manager
            .sources
            .as_ref()?
            .composition_settings
            .as_ref()?
            .read()
            .ok()
            .map(|settings| {
              if pane == 0 {
                settings.recording_output.primary.width
              } else {
                settings.recording_output.camera.width
              }
            })?;
          let width = crate::editor::screenshot_preview::hover_width_points(progress)
            * f64::from(output_width)
            / image_points;
          Some((pane, local, width as f32))
        });
      #[cfg(target_os = "windows")]
      surface
        .redraw_annotation_hover(halo.map(|(pane, local, width)| (u64::from(pane), local, width)));
      #[cfg(target_os = "macos")]
      surface.redraw_annotation_hover(halo);
    }
    let annotation_id = manager
      .annotation_targets()
      .get(index as usize)
      .map(|(_, annotation)| annotation.id.clone());
    let _ = app.emit(
      "editor://recording-annotation-hover",
      Hover {
        session_id,
        annotation_id,
      },
    );
  }));
}
