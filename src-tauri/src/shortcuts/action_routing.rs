// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The window that carries out an action the frontend owns.
///
/// The label states ownership; it does not scope delivery. The frontend's
/// `listen` registers for any target, so every window receives every one of
/// these events and each listener has to match on the action itself.
pub(super) const fn action_window(action: ShortcutAction) -> Option<WindowLabel> {
  match action {
    ShortcutAction::ToggleRecordingControls
    | ShortcutAction::StartStopRecording
    | ShortcutAction::TakeNotes => Some(WindowLabel::Island),
    // The overlay opens itself in region-edit mode and captures what the user
    // settles on, so the screenshot never goes near the recording controls.
    ShortcutAction::TakeScreenshot | ShortcutAction::TakeReferenceScreenshot => {
      Some(WindowLabel::RegionSelector)
    }
    ShortcutAction::RecognizeText => Some(WindowLabel::Island),
    ShortcutAction::AnnotateClear
    | ShortcutAction::AnnotateOverlay
    | ShortcutAction::PauseResumeRecording
    | ShortcutAction::RulerOverlay
    | ShortcutAction::SaveReplay
    | ShortcutAction::ToggleKeepAwake
    | ShortcutAction::ToggleKeepScreenOn
    | ShortcutAction::ToggleMicrophoneMute
    | ShortcutAction::Dictate => None,
  }
}

/// Actions waiting for the page that carries them out to be built. The page
/// takes its own as soon as it is listening (`take_pending_shortcut_action`):
/// sent the moment it says it is ready, the event could reach the page before
/// its listener did.
static PENDING: std::sync::Mutex<Vec<(WindowLabel, ShortcutAction)>> =
  std::sync::Mutex::new(Vec::new());

fn take_pending(window: WindowLabel) -> Option<ShortcutAction> {
  let mut pending = PENDING
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner);
  let index = pending.iter().position(|(label, _)| *label == window)?;
  Some(pending.remove(index).1)
}

pub(super) fn notify_frontend(app: &AppHandle, action: ShortcutAction) {
  let Some(window) = action_window(action) else {
    return;
  };
  if !crate::app_windows::on_demand::is_on_demand(window) {
    return dispatch(app, window, action);
  }
  {
    let mut pending = PENDING
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner);
    pending.retain(|(label, _)| *label != window);
    pending.push((window, action));
  }
  let dispatch_app = app.clone();
  crate::app_windows::on_demand::open(app, window, move |opened| {
    // A page built for the action has taken it already; one built before is
    // listening, and is told.
    let Some(action) = take_pending(window) else {
      return;
    };
    match opened {
      Ok(_) => dispatch(&dispatch_app, window, action),
      Err(error) => diagnostics::record(
        "frontend_dispatch",
        serde_json::json!({"action": action, "window": window.as_str(), "error": error}),
      ),
    }
  });
}

fn dispatch(app: &AppHandle, window: WindowLabel, action: ShortcutAction) {
  let result = app.emit_to(window.as_str(), SHORTCUT_ACTION_EVENT, action);
  diagnostics::record(
    "frontend_dispatch",
    serde_json::json!({"action": action, "window": window.as_str(), "error": result.err().map(|error| error.to_string())}),
  );
}

/// The shortcut action waiting for the asking page, which has just been
/// built for it and is now listening.
#[tauri::command]
pub fn take_pending_shortcut_action(window: tauri::WebviewWindow) -> Option<ShortcutAction> {
  let label = WindowLabel::ALL
    .iter()
    .copied()
    .find(|label| label.as_str() == window.label())?;
  let action = take_pending(label)?;
  diagnostics::record(
    "frontend_dispatch",
    serde_json::json!({"action": action, "window": label.as_str(), "error": null, "taken": true}),
  );
  Some(action)
}

pub(super) const fn preserved_capture_overlays(
  action: ShortcutAction,
) -> &'static [crate::capture_overlays::CaptureOverlay] {
  match action {
    ShortcutAction::AnnotateClear | ShortcutAction::AnnotateOverlay => {
      &[crate::capture_overlays::CaptureOverlay::Annotate]
    }
    ShortcutAction::RecognizeText => &[crate::capture_overlays::CaptureOverlay::TextRecognition],
    ShortcutAction::TakeScreenshot | ShortcutAction::TakeReferenceScreenshot => &[
      crate::capture_overlays::CaptureOverlay::Annotate,
      crate::capture_overlays::CaptureOverlay::Ruler,
    ],
    ShortcutAction::RulerOverlay => &[crate::capture_overlays::CaptureOverlay::Ruler],
    // Saving looks back at what was on screen; whatever is up stays up. Keep
    // Awake and Keep Screen On have nothing to do with what is on screen.
    ShortcutAction::SaveReplay
    | ShortcutAction::ToggleKeepAwake
    | ShortcutAction::ToggleKeepScreenOn => &[
      crate::capture_overlays::CaptureOverlay::Annotate,
      crate::capture_overlays::CaptureOverlay::Ruler,
      crate::capture_overlays::CaptureOverlay::TextRecognition,
    ],
    _ => &[],
  }
}

pub(super) const fn requires_frontend_turn(action: ShortcutAction) -> bool {
  matches!(
    action,
    ShortcutAction::ToggleRecordingControls
      | ShortcutAction::RecognizeText
      | ShortcutAction::TakeScreenshot
      | ShortcutAction::TakeReferenceScreenshot
  )
}

pub(super) fn run_action(app: &AppHandle, action: ShortcutAction) {
  if !feature_availability::action_enabled(action) {
    diagnostics::record(
      "action_blocked",
      serde_json::json!({"action": action, "reason": "feature_disabled"}),
    );
    return;
  }
  if crate::app_windows::region::is_screenshot_region_session() {
    // The borrowed Region window owns screenshot teardown. Resume the action
    // through `resume_shortcut_action` only after that later IPC turn has
    // cleared the session, so shortcuts never overlap two window graphs.
    let result = app.emit_to(
      WindowLabel::RegionSelector.as_str(),
      SCREENSHOT_SHORTCUT_REQUESTED_EVENT,
      action,
    );
    diagnostics::record(
      "screenshot_handoff",
      serde_json::json!({"action": action, "error": result.err().map(|error| error.to_string())}),
    );
    return;
  }
  if matches!(
    action,
    ShortcutAction::TakeScreenshot | ShortcutAction::TakeReferenceScreenshot
  ) {
    // A recording under way is no reason: a still is taken beside it.
    if crate::editor::focus_if_screenshot_workspace_blocked(app) {
      diagnostics::record(
        "action_blocked",
        serde_json::json!({"action": action, "reason": "capture_reserved"}),
      );
      return;
    }
    // The overlay draws on the GPU, and the device may have closed while
    // idle: it opens while the frontend takes its turn and the overlay is
    // built, rather than after.
    screenwide_gpu::open_soon(screenwide_gpu::Workload::Light);
  }
  // A menu of ours open is frozen into the picture the tool works over;
  // the action resumes in `carry_out` once it has been.
  if crate::menu_freeze::intercept(app, action) {
    return;
  }
  carry_out(app, action);
}

pub(super) fn carry_out(app: &AppHandle, action: ShortcutAction) {
  if requires_frontend_turn(action) {
    // These operations create, show, or hide window graphs. Keep that work
    // outside the native global-shortcut event cycle by handing it to the
    // persistent frontend, which enters Rust again through a Tauri command.
    notify_frontend(app, action);
    return;
  }

  // Dictation types into whatever is in front, so nothing on screen goes.
  if action == ShortcutAction::Dictate {
    crate::dictation::pressed(app);
    return;
  }
  // Muting changes what the microphone hears, nothing on screen.
  if action == ShortcutAction::ToggleMicrophoneMute {
    crate::microphone::toggle_muted(app);
    return;
  }
  // Notes listen to a call and draw nothing, so nothing on screen goes
  // either. They start from the island's choice of microphone and system
  // audio, which only its page holds.
  if action == ShortcutAction::TakeNotes {
    if crate::notes::is_taking() {
      crate::notes::stop(app);
    } else {
      notify_frontend(app, action);
    }
    return;
  }
  crate::capture_overlays::dismiss_except(app, preserved_capture_overlays(action));
  match action {
    ShortcutAction::ToggleRecordingControls
    | ShortcutAction::RecognizeText
    | ShortcutAction::TakeScreenshot
    | ShortcutAction::TakeReferenceScreenshot => {
      unreachable!("handled above")
    }
    ShortcutAction::PauseResumeRecording => {
      if matches!(
        crate::recording::snapshot(app).status,
        screenwide_capture::RecordingStatus::Recording
          | screenwide_capture::RecordingStatus::Paused
      ) {
        let _ = crate::recording::toggle_pause(app);
      }
    }
    ShortcutAction::StartStopRecording => match crate::recording::snapshot(app).status {
      screenwide_capture::RecordingStatus::Idle => notify_frontend(app, action),
      screenwide_capture::RecordingStatus::Recording
      | screenwide_capture::RecordingStatus::Paused => {
        let _ = crate::recording::stop(app);
      }
      screenwide_capture::RecordingStatus::Starting => {
        let _ = crate::recording::cancel(app);
      }
      screenwide_capture::RecordingStatus::Stopping => {}
    },
    ShortcutAction::RulerOverlay => {
      crate::ruler::toggle_detached(app);
    }
    ShortcutAction::AnnotateOverlay => {
      crate::annotate::toggle_detached(app);
    }
    ShortcutAction::AnnotateClear => {
      crate::annotate::clear(app);
    }
    ShortcutAction::SaveReplay => {
      let _ = crate::recording::replay::save(app);
    }
    ShortcutAction::ToggleKeepAwake => crate::keep_awake::toggle(app),
    ShortcutAction::ToggleKeepScreenOn => crate::keep_awake::toggle_screen_on(app),
    ShortcutAction::Dictate | ShortcutAction::TakeNotes | ShortcutAction::ToggleMicrophoneMute => {
      unreachable!("handled above")
    }
  }
}
