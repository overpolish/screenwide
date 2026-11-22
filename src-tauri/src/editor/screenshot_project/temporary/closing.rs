// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Closing the screenshot editor's window, which asks first where closing
//! would delete work kept nowhere else.

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

use super::*;
use crate::app_windows::WindowLabel;

enum Answer {
  Discard,
  Save,
  Stay,
}

/// Closes the screenshot editor as its window asks. A temporary screenshot
/// holding work kept nowhere else asks first whether to save it; any other
/// closes at once, a temporary one deleted as it goes.
pub fn close_screenshot_editor(app: &AppHandle) {
  if !open_unsaved(app) {
    super::super::super::close(app, EditorKind::Screenshot);
    return;
  }
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    let asking = app.clone();
    let answer = tauri::async_runtime::spawn_blocking(move || ask(&asking))
      .await
      .unwrap_or(Answer::Stay);
    match answer {
      Answer::Save => {
        if let Err(error) = keep_open(&app) {
          eprintln!("Could not save the screenshot: {error}");
          return;
        }
      }
      Answer::Discard => set_unsaved(&app, false),
      Answer::Stay => return,
    }
    // On the main thread, as a close from the window is: the window is put
    // away by AppKit, which does nothing asked from here.
    let closing = app.clone();
    if let Err(error) = app.run_on_main_thread(move || {
      super::super::super::close(&closing, EditorKind::Screenshot);
    }) {
      eprintln!("Could not close the screenshot editor: {error}");
    }
  });
}

fn open_unsaved(app: &AppHandle) -> bool {
  let state = app.state::<EditorState>();
  let temporary = state
    .screenshot
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .and_then(EditorArtifact::project)
    .is_some_and(|project| is_temporary(app, project));
  temporary && state.screenshot_unsaved.load(Ordering::Acquire)
}

fn ask(app: &AppHandle) -> Answer {
  let save = screenwide_i18n::t!("editor-unsaved-screenshot-save");
  let discard = screenwide_i18n::t!("editor-unsaved-screenshot-discard");
  let mut dialog = app
    .dialog()
    .message(screenwide_i18n::t!("editor-unsaved-screenshot-message"))
    .title(screenwide_i18n::t!("editor-unsaved-screenshot-title"))
    .buttons(MessageDialogButtons::YesNoCancelCustom(
      save.clone(),
      discard.clone(),
      screenwide_i18n::t!("editor-unsaved-screenshot-cancel"),
    ));
  // A sheet on the editor rather than a dialog on whichever window is first.
  if let Some(window) = app.get_webview_window(WindowLabel::EditorScreenshot.as_str()) {
    dialog = dialog.parent(&window);
  }
  match dialog.blocking_show_with_result() {
    MessageDialogResult::Custom(label) if label == save => Answer::Save,
    MessageDialogResult::Custom(label) if label == discard => Answer::Discard,
    _ => Answer::Stay,
  }
}
