// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The right press a screenshot preview session answers: on an annotation it
//! opens the annotation's menu in React, and on a bare layer the layer's.

use tauri::{AppHandle, Emitter, Manager};

use super::super::preview_platform::RecordingPreviewSurface;
use super::payloads::{ScreenshotAnnotationMenuEvent, ScreenshotLayerMenuEvent};
use super::state::ScreenshotPreviewState;

/// Installs the right press callback. The chrome names an annotation by its
/// place among every layer's published grips, and a layer by its workspace
/// order.
pub(super) fn install(app: &AppHandle, surface: &mut RecordingPreviewSurface, session_id: u64) {
  let event_app = app.clone();
  surface.set_context_menu_callback(Box::new(move |layer, annotation, x, y| {
    let Some(index) = annotation else {
      let _ = event_app.emit(
        "screenshot-preview://layer-menu",
        ScreenshotLayerMenuEvent {
          pane_index: layer,
          session_id,
          x,
          y,
        },
      );
      return;
    };
    let state = event_app.state::<ScreenshotPreviewState>();
    // Never wait for this mutex from AppKit's main thread; a menu that does
    // not open is pressed for again.
    let Ok(manager) = state.0.try_lock() else {
      return;
    };
    if manager.session_id != Some(session_id) {
      return;
    }
    let Some((pane_index, annotation_id)) = manager
      .published_annotation(index as usize)
      .map(|(pane, _, annotation)| (pane, annotation.id.clone()))
    else {
      return;
    };
    drop(manager);
    let _ = event_app.emit(
      "screenshot-preview://annotation-menu",
      ScreenshotAnnotationMenuEvent {
        annotation_id,
        pane_index,
        session_id,
        x,
        y,
      },
    );
  }));
}
