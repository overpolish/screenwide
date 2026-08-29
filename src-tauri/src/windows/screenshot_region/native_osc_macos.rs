// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex,
};

use serde::Serialize;
use tauri::{Emitter, EventTarget, WebviewWindow};

mod desktop;
mod ffi;
mod semantic;
mod state;

pub use desktop::{configure_window as configure_desktop_window, DesktopBinding};
#[cfg(test)]
use semantic::native_handle;
use semantic::{apply_phase_cursor, event_kind, payload_for, result_for};

use crate::osc::{
  controller::{ControllerEvent, RegionController},
  geometry::{Handle, Monitor, Point, Size},
  gesture::GestureKind,
};

pub const NATIVE_OSC_EVENT: &str = "screenshot-region-osc";
pub const NATIVE_OSC_LAYOUT_EVENT: &str = "screenshot-region-desktop-layout";

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SemanticStatus {
  Changed,
  Finished,
  Cancelled,
  Layout,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SemanticGesture {
  Drawing,
  Moving,
  Resizing { handle: NativeHandle },
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum NativeHandle {
  Body = 1,
  North = 2,
  South = 3,
  East = 4,
  West = 5,
  NorthEast = 6,
  NorthWest = 7,
  SouthEast = 8,
  SouthWest = 9,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct NativeRegion {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NativeOscEvent {
  pub status: SemanticStatus,
  pub gesture: Option<SemanticGesture>,
  pub region: Option<NativeRegion>,
  pub monitor_id: Option<u32>,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputPhase {
  Hover = 1,
  Down = 2,
  Drag = 3,
  Up = 4,
  Cancel = 5,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultStatus {
  None = 0,
  Changed = 1,
  Finished = 2,
  Cancelled = 3,
  Invalid = 255,
}
pub const RESULT_GESTURE_DRAWING: u8 = 1;
pub const RESULT_GESTURE_MOVING: u8 = 2;
pub const RESULT_GESTURE_RESIZING: u8 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeOscResult {
  pub status: u8,
  pub gesture: u8,
  pub handle: u8,
  pub cursor: u8,
  pub has_region: u8,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}
impl Default for NativeOscResult {
  fn default() -> Self {
    Self {
      status: 0,
      gesture: 0,
      handle: 0,
      cursor: 0,
      has_region: 0,
      x: 0.,
      y: 0.,
      width: 0.,
      height: 0.,
    }
  }
}
pub struct Context {
  pub controller: Mutex<RegionController>,
  allow_drawing: AtomicBool,
  desktop: Mutex<Option<DesktopBinding>>,
  window: WebviewWindow,
}
impl Context {
  pub fn new(window: WebviewWindow, width: f64, height: f64) -> Box<Self> {
    Box::new(Self {
      allow_drawing: AtomicBool::new(true),
      controller: Mutex::new(RegionController::new(
        Monitor {
          size: Size { width, height },
        },
        None,
        None,
      )),
      desktop: Mutex::new(None),
      window,
    })
  }

  fn input(&self, phase: u32, point: Point, shift: bool) -> NativeOscResult {
    let point = self.controller_point(point);
    let allow_drawing = self.allow_drawing.load(Ordering::Relaxed);
    if phase == InputPhase::Down as u32
      && !allow_drawing
      && self
        .controller
        .lock()
        .is_ok_and(|controller| controller.hover_kind(point) == GestureKind::Drawing)
    {
      return invalid_result();
    }
    let (mut result, event) = {
      let Ok(mut c) = self.controller.lock() else {
        return invalid_result();
      };
      let (kind, event) = match phase {
        x if x == InputPhase::Hover as u32 => (c.hover_kind(point), None),
        x if x == InputPhase::Down as u32 => (c.pointer_down(point), None),
        x if x == InputPhase::Drag as u32 => {
          let e = c.pointer_move(point, shift);
          (e.as_ref().map_or(GestureKind::Drawing, event_kind), e)
        }
        x if x == InputPhase::Up as u32 => {
          let e = c.pointer_up(point, shift);
          (e.as_ref().map_or(GestureKind::Drawing, event_kind), e)
        }
        x if x == InputPhase::Cancel as u32 => {
          let e = c.cancel();
          (e.as_ref().map_or(GestureKind::Drawing, event_kind), e)
        }
        _ => return invalid_result(),
      };
      (result_for(kind, event.as_ref()), event)
    };
    apply_phase_cursor(phase, allow_drawing, &mut result);
    if let Some(event) = event.as_ref() {
      let (projected, monitor_id) = self.project_event(*event);
      self.project_result(&mut result);
      let _ = self.window.emit_to(
        EventTarget::webview_window(self.window.label()),
        NATIVE_OSC_EVENT,
        payload_for(&projected, monitor_id),
      );
    } else if phase == InputPhase::Down as u32 {
      let gesture = match result.gesture {
        RESULT_GESTURE_DRAWING => GestureKind::Drawing,
        RESULT_GESTURE_MOVING => GestureKind::Moving,
        RESULT_GESTURE_RESIZING => GestureKind::Resizing(match result.handle {
          2 => Handle::North,
          3 => Handle::South,
          4 => Handle::East,
          5 => Handle::West,
          6 => Handle::NorthEast,
          7 => Handle::NorthWest,
          8 => Handle::SouthEast,
          9 => Handle::SouthWest,
          _ => Handle::Body,
        }),
        _ => GestureKind::Drawing,
      };
      let committed = self.controller.lock().ok().and_then(|c| c.committed());
      let down_event = ControllerEvent::Changed {
        draft: committed,
        kind: gesture,
      };
      let (projected, monitor_id) = self.project_event(down_event);
      let _ = self.window.emit_to(
        EventTarget::webview_window(self.window.label()),
        NATIVE_OSC_EVENT,
        payload_for(&projected, monitor_id),
      );
    }
    result
  }
}
pub use state::{
  claim_pointer_surface, clear_region, configure_desktop, ensure_attached, invalid_result,
  set_allow_drawing, set_aspect, set_committed, set_desktop_presented, set_exclusion_rect,
  set_input_enabled, set_magnifier_source, set_monitor, set_show_frame, set_show_handles,
};

#[cfg(test)]
mod tests;
