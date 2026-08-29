// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "macos")]
use super::native_osc_macos as native;
use serde::Deserialize;
use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::{Emitter, EventTarget, Manager};

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ExclusionRect {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
  fn screenwide_region_osc_set(
    view: *mut std::ffi::c_void,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    visible: i32,
  ) -> i32;
}

/// Synchronizes frontend workflow state into the native region OSC.
#[cfg(target_os = "macos")]
#[tauri::command]
#[expect(
  clippy::too_many_arguments,
  reason = "Tauri exposes this function as a flat, named IPC command"
)]
pub fn set_screenshot_region_osc(
  app: AppHandle,
  window: String,
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  visible: bool,
  aspect: Option<f64>,
  input_enabled: bool,
  exclusion_rect: Option<ExclusionRect>,
  show_frame: bool,
  show_handles: bool,
  allow_drawing: bool,
  monitor_width: f64,
  monitor_height: f64,
  desktop: bool,
  monitor_id: Option<u32>,
) -> Result<bool, String> {
  let target = app
    .get_webview_window(&window)
    .ok_or_else(|| format!("Screenshot overlay not found: {window}"))?;
  let desktop_anchor = desktop
    .then(|| monitor_id.ok_or_else(|| "Desktop Region OSC requires a monitor".to_owned()))
    .transpose()?;
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = target
        .ns_view()
        .map_err(|error| error.to_string())
        .and_then(|view| {
          let attached =
            native::ensure_attached(view.cast(), target.clone(), monitor_width, monitor_height);
          let rect = crate::osc::geometry::Rect {
            origin: crate::osc::geometry::Point { x, y },
            size: crate::osc::geometry::Size { width, height },
          };
          let committed = rect.committed().then_some(rect);
          let desktop_binding = desktop_anchor
            .map(|anchor| native::configure_desktop_window(view.cast(), anchor))
            .transpose()?;
          let reconciled = desktop_binding
            .as_ref()
            .and_then(|binding| committed.and_then(|region| binding.reconcile_local(region)));
          let draw_rect = reconciled
            .map(|region| region.global)
            .or_else(|| {
              desktop_binding
                .as_ref()
                .and_then(|binding| binding.project_local(rect))
            })
            .unwrap_or(rect);
          let controller_committed = desktop_binding
            .as_ref()
            .map(|_| reconciled.map(|region| region.anchor_local))
            .unwrap_or(committed);
          let layout_payload = desktop_binding.as_ref().and_then(|binding| {
            binding.layout_changed.then(|| {
              let region = reconciled.map(|region| native::NativeRegion {
                x: region.owner_local.origin.x,
                y: region.owner_local.origin.y,
                width: region.owner_local.size.width,
                height: region.owner_local.size.height,
              });
              native::NativeOscEvent {
                status: native::SemanticStatus::Layout,
                gesture: None,
                region,
                monitor_id: Some(reconciled.map_or(binding.anchor_id, |region| region.owner_id)),
              }
            })
          });
          let monitor_ready = attached
            && desktop_binding.map_or_else(
              || native::set_monitor(view.cast(), monitor_width, monitor_height),
              |binding| native::configure_desktop(view.cast(), binding, controller_committed),
            );
          if monitor_ready && visible && !desktop {
            let _ = native::set_committed(view.cast(), Some(rect));
          }
          let exclusion_rect = exclusion_rect.map(|rect| crate::osc::geometry::Rect {
            origin: crate::osc::geometry::Point {
              x: rect.x,
              y: rect.y,
            },
            size: crate::osc::geometry::Size {
              width: rect.width,
              height: rect.height,
            },
          });
          let available = monitor_ready
            && native::set_allow_drawing(view.cast(), allow_drawing)
            && native::set_aspect(view.cast(), aspect)
            && native::set_exclusion_rect(view.cast(), exclusion_rect)
            && native::set_show_frame(view.cast(), show_frame)
            && native::set_show_handles(view.cast(), show_handles)
            && native::set_input_enabled(view.cast(), input_enabled);
          let presented = unsafe {
            available
              && screenwide_region_osc_set(
                view.cast(),
                draw_rect.origin.x,
                draw_rect.origin.y,
                draw_rect.size.width,
                draw_rect.size.height,
                i32::from(visible),
              ) != 0
          };
          if presented && allow_drawing && input_enabled {
            let _ = native::claim_pointer_surface(view.cast());
          }
          if presented {
            if let Some(payload) = layout_payload {
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

#[cfg(not(target_os = "macos"))]
#[tauri::command]
#[expect(
  clippy::too_many_arguments,
  reason = "the cross-platform Tauri IPC signature must match macOS"
)]
pub fn set_screenshot_region_osc(
  _app: AppHandle,
  _window: String,
  _x: f64,
  _y: f64,
  _width: f64,
  _height: f64,
  _visible: bool,
  _aspect: Option<f64>,
  _input_enabled: bool,
  _exclusion_rect: Option<ExclusionRect>,
  _show_frame: bool,
  _show_handles: bool,
  _allow_drawing: bool,
  _monitor_width: f64,
  _monitor_height: f64,
  _desktop: bool,
  _monitor_id: Option<u32>,
) -> Result<bool, String> {
  Ok(false)
}
