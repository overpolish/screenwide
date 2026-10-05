// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::menu::{IconMenuItemBuilder, Menu, MenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Wry};

use crate::app_windows;
use crate::recording::RecordingStatus;

mod icons;
mod status;
#[cfg(target_os = "windows")]
mod windows_menu;
#[cfg(target_os = "windows")]
mod windows_theme;

const ANNOTATE_MENU_ID: &str = "annotate";
const ANNOTATE_CLEAR_MENU_ID: &str = "annotate-clear";
const DELAYED_SCREENSHOT_MENU_ID: &str = "delayed-screenshot";
const DISCARD_MENU_ID: &str = "discard-recording";
const OPEN_CLIPBOARD_SCREENSHOT_MENU_ID: &str = "open-clipboard-screenshot";
const OPEN_MENU_ID: &str = "open-screenwide";
const PAUSE_MENU_ID: &str = "pause-recording";
const QUIT_MENU_ID: &str = "quit-screenwide";
const RECOGNIZE_TEXT_MENU_ID: &str = "recognize-text";
const RULER_OVERLAY_MENU_ID: &str = "ruler-overlay";
const SETTINGS_MENU_ID: &str = "open-settings";
const STOP_MENU_ID: &str = "stop-recording";
const TRAY_ID: &str = "screenwide";

/// The recording controls join the menu only while there is a recording to
/// control. Quit always stays, because the tray is not the only way out.
fn build_menu(app: &AppHandle, status: RecordingStatus) -> tauri::Result<Menu<Wry>> {
  // While a countdown runs, the item that started it is the way to stop it.
  let delayed_screenshot = if crate::screenshots::delayed::remaining().is_some() {
    IconMenuItemBuilder::with_id(DELAYED_SCREENSHOT_MENU_ID, "Cancel Delayed Screenshot")
      .icon(icons::load(icons::CANCEL)?)
  } else {
    IconMenuItemBuilder::with_id(DELAYED_SCREENSHOT_MENU_ID, "Delayed Screenshot")
      .icon(icons::load(icons::TIMER)?)
      .enabled(status == RecordingStatus::Idle)
  }
  .build(app)?;
  let mut builder = MenuBuilder::new(app)
    .icon(OPEN_MENU_ID, "Open Screenwide", icons::load(icons::OPEN)?)
    .icon(
      OPEN_CLIPBOARD_SCREENSHOT_MENU_ID,
      "Open Screenshot from Clipboard",
      icons::load(icons::CLIPBOARD)?,
    )
    .item(&delayed_screenshot);

  if matches!(status, RecordingStatus::Recording | RecordingStatus::Paused) {
    let pause_label = if status == RecordingStatus::Paused {
      "Resume Recording"
    } else {
      "Pause Recording"
    };
    builder = builder
      .separator()
      .icon(
        PAUSE_MENU_ID,
        pause_label,
        icons::load(if status == RecordingStatus::Paused {
          icons::RESUME
        } else {
          icons::PAUSE
        })?,
      )
      .icon(STOP_MENU_ID, "Stop Recording", icons::load(icons::STOP)?)
      .icon(
        DISCARD_MENU_ID,
        "Discard Recording",
        icons::load(icons::DISCARD)?,
      );
  } else if status == RecordingStatus::Starting {
    builder = builder.separator().icon(
      DISCARD_MENU_ID,
      "Cancel Recording",
      icons::load(icons::CANCEL)?,
    );
  }

  let mut recognize_text =
    IconMenuItemBuilder::with_id(RECOGNIZE_TEXT_MENU_ID, "Recognize Text/QR")
      .icon(icons::load(icons::TEXT)?)
      .enabled(crate::text_recognition::settings::enabled());
  if let Some(shortcut) =
    crate::shortcuts::shortcut_for(app, crate::shortcuts::ShortcutAction::RecognizeText)
  {
    recognize_text = recognize_text.accelerator(shortcut);
  }
  let recognize_text = recognize_text.build(app)?;

  let mut ruler_overlay = IconMenuItemBuilder::with_id(RULER_OVERLAY_MENU_ID, "Ruler Overlay")
    .icon(icons::load(icons::RULER)?)
    .enabled(crate::ruler::settings::enabled());
  if let Some(shortcut) =
    crate::shortcuts::shortcut_for(app, crate::shortcuts::ShortcutAction::RulerOverlay)
  {
    ruler_overlay = ruler_overlay.accelerator(shortcut);
  }
  let ruler_overlay = ruler_overlay.build(app)?;

  let mut annotate = IconMenuItemBuilder::with_id(ANNOTATE_MENU_ID, "Annotate")
    .icon(icons::load(icons::ANNOTATE)?)
    .enabled(crate::annotate::settings::enabled());
  if let Some(shortcut) =
    crate::shortcuts::shortcut_for(app, crate::shortcuts::ShortcutAction::AnnotateOverlay)
  {
    annotate = annotate.accelerator(shortcut);
  }
  let annotate = annotate.build(app)?;

  // Offered only when there is something to clear, and only refreshed when
  // the overlay lets go of the screen - which is the only time this menu can
  // be reached, since the overlay covers the menu bar while it draws.
  let mut clear_annotations =
    IconMenuItemBuilder::with_id(ANNOTATE_CLEAR_MENU_ID, "Clear Annotations")
      .icon(icons::load(icons::CANCEL)?)
      .enabled(crate::annotate::has_annotations());
  if let Some(shortcut) =
    crate::shortcuts::shortcut_for(app, crate::shortcuts::ShortcutAction::AnnotateClear)
  {
    clear_annotations = clear_annotations.accelerator(shortcut);
  }
  let clear_annotations = clear_annotations.build(app)?;

  let menu = builder
    .separator()
    .item(&recognize_text)
    .item(&ruler_overlay)
    .item(&annotate)
    .item(&clear_annotations)
    .icon(SETTINGS_MENU_ID, "Settings", icons::load(icons::SETTINGS)?)
    .separator()
    .icon(QUIT_MENU_ID, "Quit Screenwide", icons::load(icons::QUIT)?)
    .build()?;
  #[cfg(target_os = "windows")]
  icons::apply_menu_style(&menu);
  Ok(menu)
}

pub fn initialize(app: &mut App) -> tauri::Result<()> {
  let menu = build_menu(app.handle(), RecordingStatus::Idle)?;

  let tray = TrayIconBuilder::with_id(TRAY_ID)
    .icon(status::icon(RecordingStatus::Idle, None)?)
    .icon_as_template(cfg!(target_os = "macos"))
    .menu(&menu)
    .show_menu_on_left_click(false)
    .tooltip(status::tooltip(RecordingStatus::Idle, None))
    .on_menu_event(|app, event| {
      let preserved: &[crate::capture_overlays::CaptureOverlay] = match event.id().as_ref() {
        ANNOTATE_CLEAR_MENU_ID | ANNOTATE_MENU_ID | DELAYED_SCREENSHOT_MENU_ID => {
          &[crate::capture_overlays::CaptureOverlay::Annotate]
        }
        RECOGNIZE_TEXT_MENU_ID => &[crate::capture_overlays::CaptureOverlay::TextRecognition],
        RULER_OVERLAY_MENU_ID => &[crate::capture_overlays::CaptureOverlay::Ruler],
        _ => &[],
      };
      crate::capture_overlays::dismiss_except(app, preserved);
      match event.id().as_ref() {
        ANNOTATE_MENU_ID => {
          crate::annotate::toggle_detached(app);
        }
        ANNOTATE_CLEAR_MENU_ID => {
          crate::annotate::clear(app);
        }
        DELAYED_SCREENSHOT_MENU_ID => {
          if crate::screenshots::delayed::remaining().is_some() {
            crate::screenshots::delayed::cancel(app);
          } else if let Err(error) = crate::screenshots::delayed::start(
            app,
            crate::screenshots::delayed::DelayedTarget::DisplayUnderPointer,
          ) {
            eprintln!("Could not start a delayed screenshot from the tray: {error}");
          }
        }
        DISCARD_MENU_ID => report("discard", crate::recording::cancel(app)),
        OPEN_CLIPBOARD_SCREENSHOT_MENU_ID => {
          crate::screenshots::open_clipboard_in_export(app);
        }
        OPEN_MENU_ID => show_main_window(app),
        PAUSE_MENU_ID => report("pause", crate::recording::toggle_pause(app)),
        QUIT_MENU_ID => app.exit(0),
        RECOGNIZE_TEXT_MENU_ID => {
          crate::text_recognition::start_detached(app);
        }
        RULER_OVERLAY_MENU_ID => {
          crate::ruler::start_detached(app);
        }
        SETTINGS_MENU_ID => {
          if let Err(error) = crate::settings::show(app) {
            eprintln!("Could not open settings from the tray: {error}");
          }
        }
        STOP_MENU_ID => report("stop", crate::recording::stop(app)),
        _ => {}
      }
    })
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        show_main_window(tray.app_handle());
      }
    })
    .build(app)?;

  #[cfg(target_os = "windows")]
  windows_menu::register(&menu);
  #[cfg(target_os = "windows")]
  windows_theme::install(&tray)?;

  #[cfg(target_os = "macos")]
  icons::apply_templates(&tray)?;
  #[cfg(not(target_os = "macos"))]
  let _ = tray;

  Ok(())
}

/// Reflects the recording state in the tray. Every piece of state is read
/// lazily inside the handlers, so nothing here depends on setup ordering.
pub fn apply_recording_status(app: &AppHandle, status: RecordingStatus) {
  let app = app.clone();
  // Transitions are driven from command threads and background tasks, while
  // menus and tray icons are main-thread objects.
  let _ = app.clone().run_on_main_thread(move || {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
      return;
    };

    apply_icon(&tray, status);

    if let Ok(menu) = build_menu(&app, status) {
      #[cfg(target_os = "windows")]
      let menu_for_registration = menu.clone();
      if tray.set_menu(Some(menu)).is_ok() {
        #[cfg(target_os = "windows")]
        windows_menu::register(&menu_for_registration);
      }
      #[cfg(target_os = "macos")]
      let _ = icons::apply_templates(&tray);
    }
  });
}

pub fn refresh(app: &AppHandle) {
  apply_recording_status(app, crate::recording::snapshot(app).status);
}

/// Updates the icon and tooltip alone, for a Delayed Screenshot's countdown.
/// Rebuilding the menu every second would close it under a user reaching for
/// Cancel.
pub fn refresh_icon(app: &AppHandle) {
  let status = crate::recording::snapshot(app).status;
  let app = app.clone();
  let _ = app.clone().run_on_main_thread(move || {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
      apply_icon(&tray, status);
    }
  });
}

/// The countdown is read here, on the main thread, rather than passed in: a
/// tick queued behind a cancellation then shows what is current, not what was.
fn apply_icon(tray: &tauri::tray::TrayIcon, status: RecordingStatus) {
  let countdown = crate::screenshots::delayed::remaining();
  if let Ok(icon) = status::icon(status, countdown) {
    let _ = tray.set_icon(Some(icon));
  }
  #[cfg(target_os = "macos")]
  let _ = tray.set_icon_as_template(true);
  let _ = tray.set_tooltip(Some(status::tooltip(status, countdown)));
}

fn report(action: &str, result: Result<(), String>) {
  if let Err(error) = result {
    eprintln!("Could not {action} the recording from the tray: {error}");
  }
}

fn show_main_window(app: &AppHandle) {
  crate::capture_overlays::dismiss_all(app);
  #[cfg(target_os = "macos")]
  if !crate::permissions::has_required_recording_permissions(app) {
    let _ = crate::permissions::show_permissions_window(app);
    return;
  }

  let _ = app_windows::show_recording_ui(app);
}
