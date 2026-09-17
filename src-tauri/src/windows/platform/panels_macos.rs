// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[cfg(target_os = "macos")]
tauri_panel! {
  // The recording bar, the source selector, the standalone listbox and the
  // region selector all take typed input at some point, so they have to be
  // able to become key - `becomes_key_only_if_needed` keeps that to the moments
  // a text field actually asks for it.
  panel!(RecordingBarPanel {
    config: {
      can_become_key_window: true,
      can_become_main_window: false,
      becomes_key_only_if_needed: true,
      hides_on_deactivate: false,
      is_floating_panel: true,
      works_when_modal: true
    }
  })

  // Quick Screenshot must own main as well as key status during activation;
  // otherwise AppKit restores a background Settings/Editor window as main.
  // Normal recording presentation remains nonactivating and never asks for main.
  panel!(RegionSelectorPanel {
    config: {
      can_become_key_window: true,
      can_become_main_window: true,
      becomes_key_only_if_needed: true,
      hides_on_deactivate: false,
      is_floating_panel: true,
      works_when_modal: true
    }
  })

  // The dock is buttons only: it never needs the keyboard, so it refuses key
  // status outright. That way nothing it does can pull keyboard focus off the
  // app the user is recording.
  panel!(RecordingDockPanel {
    config: {
      can_become_key_window: false,
      can_become_main_window: false,
      becomes_key_only_if_needed: true,
      hides_on_deactivate: false,
      is_floating_panel: true,
      works_when_modal: true
    }
  })

  // The annotate toolbar carries a field - the counter's aim - and a window
  // that cannot become key has nothing to type into. `becomes_key_only_if_needed`
  // is what keeps that to the field: pressing the toolbar's buttons leaves the
  // keyboard with the anchor host, so the letters that pick up a tool keep
  // working, and only a focused field asks for key status.
  panel!(AnnotateToolbarPanel {
    config: {
      can_become_key_window: true,
      can_become_main_window: false,
      becomes_key_only_if_needed: true,
      hides_on_deactivate: false,
      is_floating_panel: true,
      works_when_modal: true
    }
  })

  panel_event!(InactiveWebviewHoverHandler {})
}

/// WebKit suppresses page hover whenever its macOS window is not key, even
/// though AppKit continues tracking the pointer. A dedicated tracking owner
/// sends window-local positions to the page bridge without activating the
/// window, matching native AppKit control hover without taking keyboard focus.
#[cfg(target_os = "macos")]
pub(super) fn enable_inactive_webview_hover(window: &WebviewWindow) -> tauri::Result<()> {
  let content_view = window.ns_view()? as usize;
  let native_window = window.ns_window()? as usize;
  let bridge_window = window.clone();
  window.with_webview(move |_webview| unsafe {
    use objc2::{runtime::AnyObject, AnyThread};
    use objc2_app_kit::{NSTrackingArea, NSTrackingAreaOptions, NSView, NSWindow};

    unsafe extern "C" {
      fn objc_getAssociatedObject(
        object: *const AnyObject,
        key: *const std::ffi::c_void,
      ) -> *mut AnyObject;
      fn objc_setAssociatedObject(
        object: *const AnyObject,
        key: *const std::ffi::c_void,
        value: *const AnyObject,
        policy: usize,
      );
    }

    static HOVER_HANDLER_KEY: u8 = 0;
    const OBJC_ASSOCIATION_RETAIN_NONATOMIC: usize = 1;

    let content_view: &NSView = &*(content_view as *const NSView);
    let association_key = (&raw const HOVER_HANDLER_KEY).cast();
    if !objc_getAssociatedObject(
      content_view as *const NSView as *const AnyObject,
      association_key,
    )
    .is_null()
    {
      return;
    }

    let native_window: &NSWindow = &*(native_window as *const NSWindow);
    native_window.setAcceptsMouseMovedEvents(true);

    let handler = InactiveWebviewHoverHandler::new();
    let entered_window = bridge_window.clone();
    handler.on_mouse_entered(move |event| {
      let location = event.locationInWindow();
      let _ = entered_window.eval(format!(
        "globalThis.__SCREENWIDE_INACTIVE_HOVER__?.move({:.3},{:.3})",
        location.x, location.y
      ));
    });
    let exited_window = bridge_window.clone();
    handler.on_mouse_exited(move |_event| {
      let _ = exited_window.eval("globalThis.__SCREENWIDE_INACTIVE_HOVER__?.clear()");
    });
    let moved_window = bridge_window.clone();
    handler.on_mouse_moved(move |event| {
      let location = event.locationInWindow();
      let _ = moved_window.eval(format!(
        "globalThis.__SCREENWIDE_INACTIVE_HOVER__?.move({:.3},{:.3})",
        location.x, location.y
      ));
    });

    let options = NSTrackingAreaOptions::MouseEnteredAndExited
      | NSTrackingAreaOptions::MouseMoved
      | NSTrackingAreaOptions::CursorUpdate
      | NSTrackingAreaOptions::ActiveAlways
      | NSTrackingAreaOptions::InVisibleRect;
    let tracking_area = NSTrackingArea::initWithRect_options_owner_userInfo(
      NSTrackingArea::alloc(),
      content_view.bounds(),
      options,
      Some(&handler),
      None,
    );
    content_view.addTrackingArea(&tracking_area);
    objc_setAssociatedObject(
      content_view as *const NSView as *const AnyObject,
      association_key,
      (&*handler as *const InactiveWebviewHoverHandler).cast(),
      OBJC_ASSOCIATION_RETAIN_NONATOMIC,
    );
  })
}

#[cfg(target_os = "macos")]
pub(super) fn configure_panel<T: tauri_nspanel::FromWindow<tauri::Wry> + 'static>(
  window: &WebviewWindow,
  level: i32,
) -> tauri::Result<()> {
  let panel = window.to_panel::<T>()?;

  // Start transparent so conversion cannot expose a stale frame.
  panel.set_alpha_value(0.0);
  panel.set_level(PanelLevel::Custom(level).value());
  panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
  super::super::panel_presentation_macos::configure_order_animation(
    window.label(),
    panel.as_panel(),
  );
  panel.set_collection_behavior(
    CollectionBehavior::new()
      .full_screen_auxiliary()
      .can_join_all_spaces()
      .transient()
      .into(),
  );
  panel.set_hides_on_deactivate(false);
  panel.set_works_when_modal(true);
  panel.set_accepts_mouse_moved_events(true);
  enable_inactive_webview_hover(window)?;
  panel.hide();

  Ok(())
}

#[cfg(target_os = "macos")]
pub(super) fn registered_panel(window: &WebviewWindow) -> tauri::Result<PanelHandle<tauri::Wry>> {
  window
    .app_handle()
    .get_webview_panel(window.label())
    .map_err(|_| tauri::Error::WindowNotFound)
}

#[cfg(target_os = "macos")]
pub(super) fn recording_panel_level(window: &WebviewWindow) -> Option<i32> {
  match window.label() {
    "region-selector" => Some(27),
    "recording-bar" => Some(28),
    "recording-source-selector" => Some(29),
    label if label == "standalone-listbox" || label.starts_with("tool-panel-") => Some(31),
    "recording-dock" => Some(32),
    label if label == "glide" || label.starts_with("glide-space-") => Some(34),
    _ => None,
  }
}

#[cfg(target_os = "macos")]
pub(super) fn ensure_recording_panel(
  window: &WebviewWindow,
) -> tauri::Result<PanelHandle<tauri::Wry>> {
  if let Ok(panel) = registered_panel(window) {
    return Ok(panel);
  }

  let level = recording_panel_level(window).ok_or(tauri::Error::WindowNotFound)?;
  if window.label().starts_with("glide") || window.label() == "recording-dock" {
    configure_panel::<RecordingDockPanel>(window, level)?;
  } else if window.label() == "region-selector" {
    configure_panel::<RegionSelectorPanel>(window, level)?;
  } else {
    configure_panel::<RecordingBarPanel>(window, level)?;
  }
  registered_panel(window)
}
