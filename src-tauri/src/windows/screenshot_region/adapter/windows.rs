// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows surface adapter for the shared OSC runtime, a port of
//! `adapter/macos.rs` with the host HWND replacing `ns_view()`.

use tauri::{AppHandle, Emitter, EventTarget, Manager};

use super::{resolve_region_scene, RegionSceneRequest};
use crate::windows::screenshot_region::native_osc_windows as native;

/// Windows has no equivalent of the AppKit cursor-rect guard: the overlay owns
/// `WM_SETCURSOR` while it is hit-testable, so nothing has to be acquired.
pub(crate) fn acquire_quick_screenshot_cursor(_app: &AppHandle) -> Result<(), String> {
  Ok(())
}

pub(crate) fn release_quick_screenshot_cursor(_app: &AppHandle) -> Result<(), String> {
  Ok(())
}

pub(crate) fn apply_region_scene(
  app: &AppHandle,
  target: tauri::WebviewWindow,
  request: RegionSceneRequest,
) -> Result<bool, String> {
  let rect = request.rect;
  let visible = request.visible;
  let monitor_width = request.monitor_width;
  let monitor_height = request.monitor_height;
  let desktop_anchor = request.desktop_anchor;
  let owner_app = app.clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = (|| -> Result<bool, String> {
        let attached = native::ensure_attached(&target, monitor_width, monitor_height);
        // Stage 1 renders only the anchor display, but the binding still has to
        // exist: the frontend always requests desktop mode, and the region it
        // sends is anchor-local while the runtime works in the desktop plane.
        let desktop_binding = desktop_anchor
          .map(|anchor| native::configure_desktop_window(&target, anchor))
          .transpose()?;
        let owner = crate::windows::region::screenshot_region_scene_owner(&owner_app);
        let base = native::region_scene_request_base(&target, owner).unwrap_or_default();
        let resolved = resolve_region_scene(request, base, desktop_binding.as_ref());
        let monitor_ready = attached
          && desktop_binding.map_or_else(
            || native::set_monitor(&target, monitor_width, monitor_height),
            |binding| native::configure_desktop(&target, binding, resolved.controller_committed),
          );
        if monitor_ready && visible && desktop_anchor.is_none() {
          let _ = native::set_committed(&target, Some(rect));
        }
        let Some(scene) = native::reconcile_region_scene_request(&target, resolved.scene, owner)
        else {
          // A stale owner's update is silently dropped, exactly as on macOS.
          return Ok(true);
        };
        let presented = monitor_ready && native::apply_region_scene(&target, scene);
        let should_claim = presented
          && scene.visible
          && scene.interaction.allow_drawing
          && scene.interaction.input_enabled;
        if should_claim {
          let _ = native::claim_pointer_surface(&target);
        }
        if presented && owner == crate::osc::scene::RegionSceneOwner::RestoringNormal {
          crate::windows::region::finish_screenshot_region_restore();
        }
        if presented {
          if let Some(payload) = resolved.layout_event {
            let _ = target.emit_to(
              EventTarget::webview_window(target.label()),
              native::NATIVE_OSC_EVENT,
              payload,
            );
          }
        }
        Ok(presented)
      })();
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

pub(crate) fn set_desktop_presented(
  window: &tauri::WebviewWindow,
  presented: bool,
) -> tauri::Result<()> {
  let window = window.clone();
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || {
    let owner = crate::windows::region::screenshot_region_scene_owner(window.app_handle());
    let screenshot_session = owner == crate::osc::scene::RegionSceneOwner::Screenshot;
    if presented && owner == crate::osc::scene::RegionSceneOwner::Normal {
      let _ = native::restore_normal_region_scene(&window);
    }
    let scene = native::region_scene(&window);
    let scene_ready = scene.is_some_and(|scene| {
      scene.visible
        && owner.accepts_drawing(scene.interaction.allow_drawing)
        && (!screenshot_session || !scene.chrome.handles_visible)
    });
    if presented && !scene_ready {
      return;
    }
    let _ = native::set_desktop_presented(&window, presented);
    let should_claim = presented
      && scene.is_some_and(|scene| {
        scene.visible && scene.interaction.input_enabled && scene.interaction.allow_drawing
      });
    if should_claim {
      let _ = native::claim_pointer_surface(&window);
    }
  })
}

pub(crate) fn set_capture_affinity(
  window: &tauri::WebviewWindow,
  capturable: bool,
) -> tauri::Result<()> {
  if native::set_capture_affinity(window, capturable) {
    Ok(())
  } else {
    Err(std::io::Error::other("A Windows region OSC peer rejected capture affinity").into())
  }
}

pub(crate) fn prepare_for_region_restore(window: &tauri::WebviewWindow) -> tauri::Result<()> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app.run_on_main_thread(move || {
    let Some(mut scene) = native::region_scene(&window) else {
      let _ = sender.send(false);
      return;
    };
    scene.interaction.input_enabled = false;
    let _ = sender.send(native::apply_region_scene(&window, scene));
  })?;
  let _ = receiver.recv().map_err(|_| tauri::Error::WindowNotFound)?;
  Ok(())
}

pub(crate) fn prepare_for_screenshot(window: &tauri::WebviewWindow) -> tauri::Result<()> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app.run_on_main_thread(move || {
    // End native interaction before clearing the visual. A cancelled Quick
    // Screenshot must not carry a gesture or enabled input surface into the
    // next session merely because its driver WebView was hidden.
    let _ = native::set_input_enabled(&window, false);
    // Clear every surface before hiding its desktop peers. Reversing this
    // order lets the root surface submit one last persisted Region frame.
    let _ = native::clear_region(&window);
    let _ = native::set_desktop_presented(&window, false);
    let _ = sender.send(());
  })?;
  receiver.recv().map_err(|_| tauri::Error::WindowNotFound)
}

pub(crate) fn set_frame_visible(
  window: &tauri::WebviewWindow,
  visible: bool,
) -> Result<bool, String> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let _ = sender.send(native::set_show_frame(&window, visible));
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())
}

pub(crate) fn set_magnifier_source(
  window: &tauri::WebviewWindow,
  display_id: u32,
  image: crate::screenshots::CapturedImage,
) -> Result<bool, String> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let _ = sender.send(native::set_magnifier_source(
        &window,
        display_id,
        &image.rgba,
        image.width,
        image.height,
      ));
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())
}
