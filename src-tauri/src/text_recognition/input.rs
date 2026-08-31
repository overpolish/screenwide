// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral OCR routing for normalized OSC input.

use crate::osc::{
  desktop::DesktopBinding,
  geometry::{Point, Rect},
  protocol::{CursorIcon, InputModifiers, InputPhase, OscResult},
};
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::interaction::TextAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OcrControlAction {
  Cancel,
  CopyAll,
  CopyParagraph,
  Reset,
  Close,
}

pub(crate) fn control_action(phase: InputPhase) -> Option<OcrControlAction> {
  Some(match phase {
    InputPhase::OcrCancel => OcrControlAction::Cancel,
    InputPhase::OcrCopyAll => OcrControlAction::CopyAll,
    InputPhase::OcrCopyParagraph => OcrControlAction::CopyParagraph,
    InputPhase::OcrReset => OcrControlAction::Reset,
    InputPhase::OcrClose => OcrControlAction::Close,
    _ => return None,
  })
}

pub(crate) fn dispatch_control(
  window: &tauri::WebviewWindow,
  phase: InputPhase,
) -> Option<OscResult> {
  let action = control_action(phase)?;
  let app = window.app_handle().clone();
  super::qr_details::hide_without_resume(&app);
  match action {
    OcrControlAction::Cancel | OcrControlAction::Close => {
      tauri::async_runtime::spawn(async move { super::dismiss(&app) });
    }
    OcrControlAction::CopyAll => {
      tauri::async_runtime::spawn(async move {
        let _ = super::copy_all_and_dismiss(&app);
      });
    }
    OcrControlAction::CopyParagraph => {
      tauri::async_runtime::spawn(async move {
        let _ = super::copy_selection_and_dismiss(&app, true, true);
      });
    }
    OcrControlAction::Reset => {
      tauri::async_runtime::spawn(async move {
        if let Err(error) = super::start(&app).await {
          eprintln!("Could not reset text recognition: {error}");
        }
      });
    }
  }
  Some(OscResult::default())
}

pub(crate) fn text_action(phase: InputPhase, modifiers: InputModifiers) -> Option<TextAction> {
  Some(match phase {
    InputPhase::Hover => TextAction::Hover,
    InputPhase::Down => TextAction::Down {
      additive: modifiers.additive,
      double: modifiers.double_click,
    },
    InputPhase::Drag => TextAction::Drag,
    InputPhase::Up => TextAction::Up,
    InputPhase::OcrSelectAll => TextAction::SelectAll,
    InputPhase::OcrCopy => TextAction::Copy,
    _ => return None,
  })
}

pub(crate) fn selection_started(window: &tauri::WebviewWindow) {
  let _ = window.set_focus();
  super::adapter::render_window(window, super::visual::RenderPacket::default());
}

pub(crate) fn text_interaction_started(window: &tauri::WebviewWindow) {
  if !window.is_focused().unwrap_or(false) {
    let _ = window.set_focus();
  }
}

pub(crate) fn selection_finished(
  window: tauri::WebviewWindow,
  binding: DesktopBinding,
  monitor_id: u32,
  region: Rect,
) {
  super::adapter::render_window(
    &window,
    super::visual::RenderPacket::loading("Finding text and QR codes…"),
  );
  let app = window.app_handle().clone();
  tauri::async_runtime::spawn(async move {
    let capture_app = app.clone();
    let displays = binding.displays.clone();
    let selected = tauri::async_runtime::spawn_blocking(move || {
      capture_app
        .state::<super::TextRecognitionState>()
        .select_desktop_region(&displays, monitor_id, region)
    })
    .await;
    match selected {
      Ok(Ok(_)) => {}
      Ok(Err(error)) => return super::adapter::show_error(&app, &error),
      Err(error) => return super::adapter::show_error(&app, &error.to_string()),
    }
    if super::recognize_current(&app).await.is_err() {
      return;
    }
    if app.get_webview_window(window.label()).is_none() {
      return;
    }
    let main_app = app.clone();
    let _ = app.run_on_main_thread(move || {
      if let Some(target) = main_app.get_webview_window(window.label()) {
        let _ = target.set_ignore_cursor_events(false);
        let _ = crate::windows::show(&target, true);
      }
    });
  });
}

pub(crate) fn dispatch_text_input(
  window: &tauri::WebviewWindow,
  phase: InputPhase,
  point: Point,
  modifiers: u8,
  display_id: Option<u32>,
) -> OscResult {
  let Some(action) = text_action(phase, InputModifiers::from_bits(modifiers)) else {
    return crate::osc::runtime::invalid_result();
  };
  let app = window.app_handle();
  let Some(update) = app
    .state::<super::TextRecognitionState>()
    .text_input(action, point)
  else {
    return crate::osc::runtime::invalid_result();
  };
  if let Some(snapshot) = update.snapshot {
    super::adapter::render_window(window, super::visual::RenderPacket::ready(&snapshot));
  }
  if let Some(text) = update.copy_text {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
      let _ = app.clipboard().write_text(text);
      super::dismiss(&app);
    });
  }
  if let Some(code) = update.qr_code {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
      if let Err(error) = super::qr_details::show(&app, code, display_id) {
        eprintln!("Could not show QR details: {error}");
      }
    });
  }
  OscResult {
    cursor: if update.qr_cursor {
      CursorIcon::PointingHand as u8
    } else if update.text_cursor {
      CursorIcon::IBeam as u8
    } else {
      CursorIcon::Arrow as u8
    },
    ..Default::default()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shortcut_and_pointer_actions_share_the_normalized_protocol() {
    assert_eq!(
      text_action(InputPhase::OcrSelectAll, InputModifiers::default()),
      Some(TextAction::SelectAll)
    );
    assert_eq!(
      text_action(InputPhase::OcrCopy, InputModifiers::default()),
      Some(TextAction::Copy)
    );
    assert_eq!(
      text_action(
        InputPhase::Down,
        InputModifiers {
          additive: true,
          double_click: true,
          ..Default::default()
        },
      ),
      Some(TextAction::Down {
        additive: true,
        double: true,
      })
    );
  }
}
