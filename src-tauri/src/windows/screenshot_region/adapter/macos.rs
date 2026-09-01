// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Emitter, EventTarget, Manager};

use super::{resolve_region_scene, RegionSceneRequest};
use crate::windows::screenshot_region::native_osc_macos as native;

pub(crate) fn acquire_quick_screenshot_cursor(app: &AppHandle) -> Result<(), String> {
  crate::osc::cursor::macos::acquire_quick_screenshot(app)
}

pub(crate) fn release_quick_screenshot_cursor(app: &AppHandle) -> Result<(), String> {
  crate::osc::cursor::macos::release_quick_screenshot(app)
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
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = target
        .ns_view()
        .map_err(|error| error.to_string())
        .and_then(|view| {
          let attached =
            native::ensure_attached(view.cast(), target.clone(), monitor_width, monitor_height);
          let desktop_binding = desktop_anchor
            .map(|anchor| native::configure_desktop_window(view.cast(), anchor))
            .transpose()?;
          let owner = crate::windows::region::screenshot_region_scene_owner();
          let base = native::region_scene_request_base(view.cast(), owner).unwrap_or_default();
          let resolved = resolve_region_scene(request, base, desktop_binding.as_ref());
          let monitor_ready = attached
            && desktop_binding.map_or_else(
              || native::set_monitor(view.cast(), monitor_width, monitor_height),
              |binding| {
                native::configure_desktop(view.cast(), binding, resolved.controller_committed)
              },
            );
          if monitor_ready && visible && desktop_anchor.is_none() {
            let _ = native::set_committed(view.cast(), Some(rect));
          }
          let Some(scene) =
            native::reconcile_region_scene_request(view.cast(), resolved.scene, owner)
          else {
            return Ok(true);
          };
          let presented = monitor_ready && native::apply_region_scene(view.cast(), scene);
          let should_claim = presented
            && scene.visible
            && scene.interaction.allow_drawing
            && scene.interaction.input_enabled;
          if should_claim {
            let _ = native::claim_pointer_surface(view.cast());
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
        });
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
    if let Ok(view) = window.ns_view() {
      let owner = crate::windows::region::screenshot_region_scene_owner();
      let screenshot_session = owner == crate::osc::scene::RegionSceneOwner::Screenshot;
      if presented && owner == crate::osc::scene::RegionSceneOwner::Normal {
        let _ = native::restore_normal_region_scene(view.cast());
      }
      let scene = native::region_scene(view.cast());
      let scene_ready = scene.is_some_and(|scene| {
        scene.visible
          && owner.accepts_drawing(scene.interaction.allow_drawing)
          && (!screenshot_session || !scene.chrome.handles_visible)
      });
      if presented && !scene_ready {
        return;
      }
      let _ = native::set_desktop_presented(view.cast(), presented);
      let should_claim = presented
        && scene.is_some_and(|scene| {
          scene.visible && scene.interaction.input_enabled && scene.interaction.allow_drawing
        });
      if should_claim {
        let _ = native::claim_pointer_surface(view.cast());
      }
    }
  })
}

pub(crate) fn prepare_for_region_restore(window: &tauri::WebviewWindow) -> tauri::Result<()> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app.run_on_main_thread(move || {
    let result = window.ns_view().map(|view| {
      let Some(mut scene) = native::region_scene(view.cast()) else {
        return false;
      };
      scene.interaction.input_enabled = false;
      native::apply_region_scene(view.cast(), scene)
    });
    let _ = sender.send(result);
  })?;
  receiver
    .recv()
    .unwrap_or(Err(tauri::Error::WindowNotFound))?;
  Ok(())
}

pub(crate) fn prepare_for_screenshot(window: &tauri::WebviewWindow) -> tauri::Result<()> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app.run_on_main_thread(move || {
    let result = window.ns_view().map(|view| {
      // Clear every surface before hiding its desktop peers. Reversing this
      // order lets the root surface submit one last persisted Region frame.
      let _ = native::clear_region(view.cast());
      let _ = native::set_desktop_presented(view.cast(), false);
    });
    let _ = sender.send(result);
  })?;
  receiver.recv().unwrap_or(Err(tauri::Error::WindowNotFound))
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
      let result = window
        .ns_view()
        .map(|view| native::set_show_frame(view.cast(), visible))
        .map_err(|error| error.to_string());
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

pub(crate) fn set_magnifier_source(
  window: &tauri::WebviewWindow,
  image: crate::screenshots::CapturedImage,
) -> Result<bool, String> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = window
        .ns_view()
        .map(|view| {
          native::set_magnifier_source(view.cast(), &image.rgba, image.width, image.height)
        })
        .map_err(|error| error.to_string());
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}
