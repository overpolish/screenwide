// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared runtime for Region, Text Recognition, and Ruler OSC hosts.

mod desktop;

pub use desktop::project_desktop_event;

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex,
};

use tauri::{Emitter, EventTarget, Manager, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::{
  controller::{ControllerEvent, RegionController},
  desktop::DesktopBinding,
  geometry::{Monitor, Point, Size},
  protocol::{CursorIcon, InputModifiers, InputPhase, OscResult, Purpose, ResultStatus},
  scene::RegionSceneState,
  semantic::{event_payload, REGION_EVENT},
  session::dispatch_region,
};

pub fn invalid_result() -> OscResult {
  OscResult {
    status: ResultStatus::Invalid as u8,
    ..Default::default()
  }
}

pub struct OscRuntime {
  pub(crate) controller: Mutex<RegionController>,
  pub(crate) allow_drawing: AtomicBool,
  pub(crate) completed: AtomicBool,
  pub(crate) desktop: Mutex<Option<DesktopBinding>>,
  pub(crate) purpose: Purpose,
  pub(crate) scene: Mutex<RegionSceneState>,
  pub(crate) window: WebviewWindow,
}

impl OscRuntime {
  pub fn new(window: WebviewWindow, width: f64, height: f64, purpose: Purpose) -> Box<Self> {
    Box::new(Self {
      allow_drawing: AtomicBool::new(true),
      completed: AtomicBool::new(false),
      controller: Mutex::new(RegionController::new(
        Monitor {
          size: Size { width, height },
        },
        None,
        None,
      )),
      desktop: Mutex::new(None),
      purpose,
      scene: Mutex::new(RegionSceneState::default()),
      window,
    })
  }

  pub(crate) fn input(&self, phase: u32, point: Point, modifiers: u8) -> OscResult {
    let Some(input_phase) = InputPhase::from_raw(phase) else {
      return invalid_result();
    };
    if self.purpose == Purpose::Ruler {
      let state = self.window.app_handle().state::<crate::ruler::RulerState>();
      let Some(dispatch) = crate::ruler::dispatch_input(
        &state,
        input_phase,
        point,
        InputModifiers::from_bits(modifiers),
      ) else {
        return invalid_result();
      };
      return self.ruler_visual_result(
        &state,
        dispatch.visual,
        dispatch.copy,
        dispatch.cursor,
        dispatch.handle,
      );
    }
    if self.purpose == Purpose::TextRecognition && input_phase == InputPhase::Down {
      crate::text_recognition::qr_details::hide_without_resume(self.window.app_handle());
    }
    if self.purpose == Purpose::TextRecognition {
      if let Some(result) = crate::text_recognition::dispatch_control(&self.window, input_phase) {
        return result;
      }
    }
    if self.purpose == Purpose::TextRecognition && self.completed.load(Ordering::Acquire) {
      if input_phase == InputPhase::Down {
        crate::text_recognition::native_text_interaction_started(&self.window);
      }
      let display_id = self.desktop.lock().ok().and_then(|desktop| {
        desktop
          .as_ref()
          .and_then(|binding| binding.display_at(point))
      });
      return crate::text_recognition::native_text_input(
        &self.window,
        input_phase,
        point,
        modifiers,
        display_id,
      );
    }
    if self.purpose == Purpose::TextRecognition && input_phase == InputPhase::Down {
      crate::text_recognition::native_selection_started(&self.window);
    }
    let allow_drawing = self.allow_drawing.load(Ordering::Relaxed);
    let dispatch = {
      let Ok(mut controller) = self.controller.lock() else {
        return invalid_result();
      };
      dispatch_region(
        &mut controller,
        input_phase,
        point,
        InputModifiers::from_bits(modifiers),
        allow_drawing,
      )
    };
    let Some(dispatch) = dispatch else {
      return invalid_result();
    };
    let mut result = dispatch.result;
    if let Some(event) = dispatch.event.as_ref() {
      if self.purpose == Purpose::TextRecognition
        && matches!(
          event,
          ControllerEvent::Finished {
            committed: Some(_),
            ..
          }
        )
      {
        self.completed.store(true, Ordering::Release);
      }
      let (projected, monitor_id) = self.project_event(*event);
      self.dispatch_event(*event, projected, monitor_id);
    } else if input_phase == InputPhase::Down && self.purpose == Purpose::Region {
      let committed = self.controller.lock().ok().and_then(|c| c.committed());
      let event = ControllerEvent::Changed {
        draft: committed,
        kind: dispatch.gesture,
      };
      let (projected, monitor_id) = self.project_event(event);
      self.emit_region_event(&projected, monitor_id);
    }
    if self.purpose == Purpose::TextRecognition
      && input_phase == InputPhase::Cancel
      && result.status == ResultStatus::None as u8
    {
      result.cursor = CursorIcon::Arrow as u8;
      let app = self.window.app_handle().clone();
      tauri::async_runtime::spawn(async move {
        crate::text_recognition::dismiss(&app);
      });
    }
    result
  }

  fn ruler_visual_result(
    &self,
    state: &crate::ruler::RulerState,
    visual: crate::ruler::snapshot::RulerVisual,
    copy: Option<String>,
    cursor: CursorIcon,
    handle: u8,
  ) -> OscResult {
    if let Some(text) = copy {
      let _ = self.window.app_handle().clipboard().write_text(text);
    }
    crate::ruler::visual_result(state, visual, cursor, handle)
  }

  pub(crate) fn ruler_viewport_input(
    &self,
    display_id: u32,
    operation: u32,
    anchor: Point,
    delta: Point,
  ) -> OscResult {
    if self.purpose != Purpose::Ruler {
      return invalid_result();
    }
    let state = self.window.app_handle().state::<crate::ruler::RulerState>();
    let Some(visual) =
      crate::ruler::dispatch_viewport(&state, display_id, operation, anchor, delta)
    else {
      return invalid_result();
    };
    self.ruler_visual_result(&state, visual, None, CursorIcon::Crosshair, 0)
  }

  pub(crate) fn ruler_label_input(
    &self,
    operation: u32,
    kind: u8,
    id: u64,
    pointer: Point,
    label_center: Point,
  ) -> OscResult {
    if self.purpose != Purpose::Ruler {
      return invalid_result();
    }
    let ruler = self.window.app_handle().state::<crate::ruler::RulerState>();
    crate::ruler::dispatch_label(&ruler, operation, kind, id, pointer, label_center).map_or_else(
      invalid_result,
      |dispatch| {
        self.ruler_visual_result(
          &ruler,
          dispatch.visual,
          dispatch.copy,
          dispatch.cursor,
          dispatch.handle,
        )
      },
    )
  }

  pub(crate) fn project_event(&self, event: ControllerEvent) -> (ControllerEvent, Option<u32>) {
    let Ok(desktop) = self.desktop.lock() else {
      return (event, None);
    };
    let Some(binding) = desktop.as_ref() else {
      return (event, None);
    };
    project_desktop_event(binding, event)
  }

  fn emit_region_event(&self, event: &ControllerEvent, monitor_id: Option<u32>) {
    let _ = self.window.emit_to(
      EventTarget::webview_window(self.window.label()),
      REGION_EVENT,
      event_payload(event, monitor_id),
    );
  }

  fn dispatch_event(
    &self,
    raw: ControllerEvent,
    projected: ControllerEvent,
    monitor_id: Option<u32>,
  ) {
    if self.purpose == Purpose::Region {
      self.emit_region_event(&projected, monitor_id);
      return;
    }
    match (raw, projected, monitor_id) {
      (
        ControllerEvent::Finished {
          committed: Some(_), ..
        },
        ControllerEvent::Finished {
          committed: Some(region),
          ..
        },
        Some(monitor_id),
      ) => {
        let binding = self.desktop.lock().ok().and_then(|binding| binding.clone());
        if let Some(binding) = binding {
          crate::text_recognition::native_selection_finished(
            self.window.clone(),
            binding,
            monitor_id,
            region,
          );
        }
      }
      (ControllerEvent::Cancelled { .. }, _, _) => {
        let app = self.window.app_handle().clone();
        tauri::async_runtime::spawn(async move {
          crate::text_recognition::dismiss(&app);
        });
      }
      _ => {}
    }
  }
}
