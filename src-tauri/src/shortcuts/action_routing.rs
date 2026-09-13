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
    ShortcutAction::ToggleRecordingBar | ShortcutAction::StartStopRecording => {
      Some(WindowLabel::RecordingBar)
    }
    // The overlay opens itself in region-edit mode and captures what the user
    // settles on, so the screenshot never goes near the recording bar.
    ShortcutAction::TakeScreenshot | ShortcutAction::TakeScreenshotToClipboard => {
      Some(WindowLabel::RegionSelector)
    }
    ShortcutAction::RecognizeText => Some(WindowLabel::RecordingBar),
    ShortcutAction::PauseResumeRecording | ShortcutAction::RulerOverlay => None,
  }
}

pub(super) fn notify_frontend(app: &AppHandle, action: ShortcutAction) {
  if let Some(window) = action_window(action) {
    let result = app.emit_to(window.as_str(), SHORTCUT_ACTION_EVENT, action);
    diagnostics::record(
      "frontend_dispatch",
      serde_json::json!({"action": action, "window": window.as_str(), "error": result.err().map(|error| error.to_string())}),
    );
  }
}

pub(super) const fn preserved_capture_overlay(
  action: ShortcutAction,
) -> Option<crate::capture_overlays::CaptureOverlay> {
  match action {
    ShortcutAction::RecognizeText => Some(crate::capture_overlays::CaptureOverlay::TextRecognition),
    ShortcutAction::TakeScreenshot
    | ShortcutAction::TakeScreenshotToClipboard
    | ShortcutAction::RulerOverlay => Some(crate::capture_overlays::CaptureOverlay::Ruler),
    _ => None,
  }
}

pub(super) const fn requires_frontend_turn(action: ShortcutAction) -> bool {
  matches!(
    action,
    ShortcutAction::ToggleRecordingBar
      | ShortcutAction::RecognizeText
      | ShortcutAction::TakeScreenshot
      | ShortcutAction::TakeScreenshotToClipboard
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
  if crate::windows::region::is_screenshot_region_session() {
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
    ShortcutAction::TakeScreenshot | ShortcutAction::TakeScreenshotToClipboard
  ) {
    let reason = if !crate::recording::is_idle(app) {
      Some("recording_not_idle")
    } else if crate::editor::focus_if_screenshot_workspace_blocked(app) {
      Some("capture_reserved")
    } else {
      None
    };
    if let Some(reason) = reason {
      diagnostics::record(
        "action_blocked",
        serde_json::json!({"action": action, "reason": reason}),
      );
      return;
    }
  }
  if requires_frontend_turn(action) {
    // These operations create, show, or hide window graphs. Keep that work
    // outside the native global-shortcut event cycle by handing it to the
    // persistent frontend, which enters Rust again through a Tauri command.
    notify_frontend(app, action);
    return;
  }

  crate::capture_overlays::dismiss_except(app, preserved_capture_overlay(action));
  match action {
    ShortcutAction::ToggleRecordingBar
    | ShortcutAction::RecognizeText
    | ShortcutAction::TakeScreenshot
    | ShortcutAction::TakeScreenshotToClipboard => {
      unreachable!("handled above")
    }
    ShortcutAction::PauseResumeRecording => {
      if matches!(
        crate::recording::snapshot(app).status,
        crate::recording::RecordingStatus::Recording | crate::recording::RecordingStatus::Paused
      ) {
        let _ = crate::recording::toggle_pause(app);
      }
    }
    ShortcutAction::StartStopRecording => match crate::recording::snapshot(app).status {
      crate::recording::RecordingStatus::Idle => {
        if !crate::editor::focus_pending_workspace(app) {
          notify_frontend(app, action);
        }
      }
      crate::recording::RecordingStatus::Recording | crate::recording::RecordingStatus::Paused => {
        let _ = crate::recording::stop(app);
      }
      crate::recording::RecordingStatus::Starting => {
        let _ = crate::recording::cancel(app);
      }
      crate::recording::RecordingStatus::Stopping => {}
    },
    ShortcutAction::RulerOverlay => {
      crate::ruler::start_detached(app);
    }
  }
}
