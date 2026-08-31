// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use serde::Serialize;
use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex,
};
use tauri::{Emitter, EventTarget, Manager, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
mod desktop;
mod ffi;
mod ocr;
mod purpose;
mod semantic;
mod state;
use crate::osc::{
  controller::{ControllerEvent, RegionController},
  geometry::{Handle, Monitor, Point, Size},
  gesture::GestureKind,
};
pub use desktop::{configure_window as configure_desktop_window, DesktopBinding};
pub use purpose::Purpose;
#[cfg(test)]
use semantic::native_handle;
use semantic::{apply_phase_cursor, event_kind, payload_for, result_for};
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
  OcrCancel = 8,
  OcrCopyAll = 9,
  OcrCopyParagraph = 10,
  OcrReset = 11,
  OcrClose = 12,
  RulerToggleCrosshair = 13,
  RulerCopyColour = 14,
  RulerAnimationFrame = 15,
  RulerDeleteMeasurement = 16,
  RulerCopyMeasurement = 17,
  RulerUndo = 18,
  RulerRedo = 19,
  RulerBeginHorizontalRange = 20,
  RulerBeginVerticalRange = 21,
  RulerFinishRange = 22,
  RulerCancelRange = 23,
  RulerHoverProbeLabel = 24,
  RulerHoverMeasurementLabel = 25,
  RulerBeginVerticalGuide = 26,
  RulerBeginHorizontalGuide = 27,
  RulerCancelGuide = 28,
  RulerCycleTolerance = 29,
  RulerSetOptionActive = 30,
  RulerBeginRadius = 31,
  RulerCancelRadius = 32,
  RulerToggleCenterlines = 33,
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
pub const RESULT_GESTURE_RESIZING: u8 = 3; // Shared FFI gesture tag.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
  pub ruler_color: u32,
  pub ruler_flags: u8,
  pub ruler_padding: [u8; 3],
}
pub struct Context {
  pub controller: Mutex<RegionController>,
  allow_drawing: AtomicBool,
  completed: AtomicBool,
  desktop: Mutex<Option<DesktopBinding>>,
  purpose: Purpose,
  window: WebviewWindow,
}
impl Context {
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
      window,
    })
  }

  fn input(&self, phase: u32, point: Point, modifiers: u8) -> NativeOscResult {
    if self.purpose == Purpose::Ruler {
      let state = self.window.app_handle().state::<crate::ruler::RulerState>();
      if matches!(
        phase,
        value if value == InputPhase::Hover as u32
          || value == InputPhase::Down as u32
          || value == InputPhase::Drag as u32
          || value == InputPhase::Up as u32
      ) {
        let _ = state.set_option_active(modifiers & 8 != 0);
      }
      let (visual, copy) = match phase {
        value if value == InputPhase::Hover as u32 => (
          state
            .map_pointer(point)
            .and_then(|value| state.hover(value)),
          None,
        ),
        value if value == InputPhase::Down as u32 => (
          state
            .map_pointer(point)
            .and_then(|value| state.pointer_down(value)),
          None,
        ),
        value if value == InputPhase::Drag as u32 => (
          state
            .map_pointer(point)
            .and_then(|value| state.pointer_drag(value)),
          None,
        ),
        value if value == InputPhase::Up as u32 => (
          state
            .map_pointer(point)
            .and_then(|value| state.pointer_up(value)),
          None,
        ),
        value if value == InputPhase::Cancel as u32 => (state.cancel_pointer(), None),
        x if x == InputPhase::RulerToggleCrosshair as u32 => (state.toggle_crosshair(), None),
        x if x == InputPhase::RulerCopyColour as u32 => state
          .copy_colour()
          .map_or((None, None), |(visual, text)| (Some(visual), Some(text))),
        x if x == InputPhase::RulerAnimationFrame as u32 => (state.animation_frame(), None),
        x if x == InputPhase::RulerDeleteMeasurement as u32 => {
          (state.delete_targeted_artifact(), None)
        }
        x if x == InputPhase::RulerCopyMeasurement as u32 => state
          .copy_latest_artifact()
          .map_or((None, None), |(visual, text)| (Some(visual), Some(text))),
        x if x == InputPhase::RulerUndo as u32 => (state.undo(), None),
        x if x == InputPhase::RulerRedo as u32 => (state.redo(), None),
        x if x == InputPhase::RulerBeginHorizontalRange as u32 => (
          state.begin_range(crate::ruler::snapshot::RangeAxis::Horizontal),
          None,
        ),
        x if x == InputPhase::RulerBeginVerticalRange as u32 => (
          state.begin_range(crate::ruler::snapshot::RangeAxis::Vertical),
          None,
        ),
        x if x == InputPhase::RulerFinishRange as u32 => (state.finish_range(), None),
        x if x == InputPhase::RulerCancelRange as u32 => (state.cancel_range(), None),
        x if x == InputPhase::RulerHoverProbeLabel as u32 => {
          (state.hover_probe_label(point.x.max(0.0) as u64), None)
        }
        x if x == InputPhase::RulerHoverMeasurementLabel as u32 => {
          (state.hover_measurement_label(point.x.max(0.0) as u64), None)
        }
        x if x == InputPhase::RulerBeginVerticalGuide as u32 => (
          state.begin_guide(crate::ruler::snapshot::GuideAxis::Vertical),
          None,
        ),
        x if x == InputPhase::RulerBeginHorizontalGuide as u32 => (
          state.begin_guide(crate::ruler::snapshot::GuideAxis::Horizontal),
          None,
        ),
        x if x == InputPhase::RulerCancelGuide as u32 => (state.cancel_guide(), None),
        x if x == InputPhase::RulerCycleTolerance as u32 => (state.cycle_tolerance(), None),
        x if x == InputPhase::RulerSetOptionActive as u32 => {
          (state.set_option_active(modifiers & 8 != 0), None)
        }
        x if x == InputPhase::RulerBeginRadius as u32 => (state.begin_radius(), None),
        x if x == InputPhase::RulerCancelRadius as u32 => (state.cancel_radius(), None),
        x if x == InputPhase::RulerToggleCenterlines as u32 => (state.toggle_centerlines(), None),
        _ => return state::invalid_result(),
      };
      let Some(visual) = visual else {
        return state::invalid_result();
      };
      let mut result = self.ruler_visual_result(&state, visual, copy);
      if phase == InputPhase::RulerHoverProbeLabel as u32
        || phase == InputPhase::RulerHoverMeasurementLabel as u32
      {
        result.cursor = ffi::CURSOR_OPEN_HAND;
      } else if let Some(axis) = state.hovered_guide_axis() {
        match axis {
          crate::ruler::snapshot::GuideAxis::Vertical => {
            result.cursor = ffi::CURSOR_HORIZONTAL;
            result.handle = 4;
          }
          crate::ruler::snapshot::GuideAxis::Horizontal => {
            result.cursor = ffi::CURSOR_VERTICAL;
            result.handle = 2;
          }
        }
      }
      return result;
    }
    if self.purpose == Purpose::TextRecognition && phase == InputPhase::Down as u32 {
      crate::text_recognition::qr_details::hide_without_resume(self.window.app_handle());
    }
    if let Some(result) = ocr::control_input(self, phase) {
      return result;
    }
    if self.purpose == Purpose::TextRecognition && self.completed.load(Ordering::Acquire) {
      if phase == InputPhase::Down as u32 {
        // Route a peer's first interaction to the Tauri owner so Command+A/C
        // has one stable key window regardless of the display clicked.
        crate::text_recognition::native_text_interaction_started(&self.window);
      }
      let display_id = self.desktop.lock().ok().and_then(|desktop| {
        desktop
          .as_ref()
          .and_then(|binding| binding.display_at(point))
      });
      return crate::text_recognition::native_text_input(
        &self.window,
        phase,
        point,
        modifiers,
        display_id,
      );
    }
    let point = self.controller_point(point);
    if self.purpose == Purpose::TextRecognition && phase == InputPhase::Down as u32 {
      crate::text_recognition::native_selection_started(&self.window);
    }
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
          let e = c.pointer_move(point, modifiers & 1 != 0);
          (e.as_ref().map_or(GestureKind::Drawing, event_kind), e)
        }
        x if x == InputPhase::Up as u32 => {
          let e = c.pointer_up(point, modifiers & 1 != 0);
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
      self.project_result(&mut result);
      self.dispatch_event(*event, projected, monitor_id);
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
      if self.purpose == Purpose::Region {
        let _ = self.window.emit_to(
          EventTarget::webview_window(self.window.label()),
          NATIVE_OSC_EVENT,
          payload_for(&projected, monitor_id),
        );
      }
    }
    result
  }

  fn ruler_visual_result(
    &self,
    state: &crate::ruler::RulerState,
    visual: crate::ruler::snapshot::RulerVisual,
    copy: Option<String>,
  ) -> NativeOscResult {
    if let Some(text) = copy {
      let _ = self.window.app_handle().clipboard().write_text(text);
    }
    let tolerance = state.tolerance_notice();
    let tolerance_mode = tolerance.map_or(0, |value| match value {
      crate::ruler::snapshot::Tolerance::ClearEdges => 1,
      crate::ruler::snapshot::Tolerance::Balanced => 2,
      crate::ruler::snapshot::Tolerance::SubtleEdges => 3,
    });
    NativeOscResult {
      status: ResultStatus::Changed as u8,
      cursor: ffi::CURSOR_CROSSHAIR,
      x: visual.screen_point.x,
      y: visual.screen_point.y,
      ruler_color: visual.packed_rgba(),
      ruler_flags: 1
        | u8::from(visual.crosshair) << 1
        | u8::from(visual.copied) << 2
        | u8::from(tolerance.is_some()) << 3
        | tolerance_mode << 4
        | u8::from(state.interaction_active()) << 6,
      ..Default::default()
    }
  }

  fn ruler_viewport_input(
    &self,
    display_id: u32,
    operation: u32,
    anchor: Point,
    delta: Point,
  ) -> NativeOscResult {
    if self.purpose != Purpose::Ruler {
      return state::invalid_result();
    }
    let action = match operation {
      1 => crate::ruler::snapshot::ViewportAction::Zoom {
        anchor,
        factor: delta.x,
      },
      2 => crate::ruler::snapshot::ViewportAction::Pan { anchor, delta },
      3 => crate::ruler::snapshot::ViewportAction::Reset { anchor },
      _ => return state::invalid_result(),
    };
    let state = self.window.app_handle().state::<crate::ruler::RulerState>();
    let Some(visual) = state.update_viewport(display_id, action) else {
      return state::invalid_result();
    };
    self.ruler_visual_result(&state, visual, None)
  }

  fn ruler_label_input(
    &self,
    operation: u32,
    kind: u8,
    id: u64,
    pointer: Point,
    label_center: Point,
  ) -> NativeOscResult {
    if self.purpose != Purpose::Ruler {
      return state::invalid_result();
    }
    let ruler = self.window.app_handle().state::<crate::ruler::RulerState>();
    let label_kind = match kind {
      1 => Some(crate::ruler::snapshot::LabelKind::Measurement),
      2 => Some(crate::ruler::snapshot::LabelKind::Probe),
      3 => Some(crate::ruler::snapshot::LabelKind::GuideGap),
      4 => Some(crate::ruler::snapshot::LabelKind::Radius),
      _ => None,
    };
    let visual = match operation {
      1 => label_kind.and_then(|kind| {
        ruler
          .map_pointer(pointer)
          .zip(ruler.map_pointer(label_center))
          .and_then(|(pointer, center)| ruler.begin_label_drag(kind, id, pointer, center))
      }),
      2 => ruler
        .map_pointer(pointer)
        .and_then(|pointer| ruler.update_label_drag(pointer)),
      3 => ruler
        .map_pointer(pointer)
        .and_then(|pointer| ruler.finish_label_drag(pointer)),
      4 => ruler.cancel_label_drag(),
      5 => label_kind.and_then(|kind| ruler.hide_label(kind, id)),
      6 => ruler
        .map_pointer(pointer)
        .and_then(|pointer| ruler.toggle_label_at(pointer)),
      7 => match label_kind {
        Some(crate::ruler::snapshot::LabelKind::Measurement) => ruler.hover_measurement_label(id),
        Some(crate::ruler::snapshot::LabelKind::Probe) => ruler.hover_probe_label(id),
        Some(crate::ruler::snapshot::LabelKind::GuideGap) => ruler.hover_guide_gap_label(id),
        Some(crate::ruler::snapshot::LabelKind::Radius) => ruler.hover_radius_label(id),
        None => None,
      },
      _ => None,
    };
    visual.map_or_else(state::invalid_result, |visual| {
      let mut result = self.ruler_visual_result(&ruler, visual, None);
      result.cursor = match operation {
        1 | 2 => ffi::CURSOR_CLOSED_HAND,
        3 | 7 => ffi::CURSOR_OPEN_HAND,
        _ => ffi::CURSOR_CROSSHAIR,
      };
      result
    })
  }

  fn dispatch_event(
    &self,
    raw: ControllerEvent,
    projected: ControllerEvent,
    monitor_id: Option<u32>,
  ) {
    if self.purpose == Purpose::Region {
      let _ = self.window.emit_to(
        EventTarget::webview_window(self.window.label()),
        NATIVE_OSC_EVENT,
        payload_for(&projected, monitor_id),
      );
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
pub use ffi::NativeOcrRect;
pub use ocr::{
  ready_result as text_recognition_ready_result, reset_input as reset_text_recognition_input,
  set_cancel_visible as set_ocr_cancel_visible, set_ocr,
};
pub use state::{
  claim_pointer_surface, clear_region, configure_desktop, ensure_attached, ensure_ruler_attached,
  ensure_text_recognition_attached, invalid_result, present_region, refresh_ruler_pointer,
  set_allow_drawing, set_aspect, set_committed, set_desktop_presented, set_exclusion_rect,
  set_input_enabled, set_magnifier_source, set_monitor, set_ruler_transient_chrome, set_show_frame,
  set_show_handles, set_snapshot, set_snapshot_composited, set_snapshot_presented,
};

#[cfg(test)]
mod tests;
