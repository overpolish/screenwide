// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows' half of a Glide session: the captured HWND, the hidden system
//! cursor and the monitor picker, under the shared session flow in
//! `glide::session`. Coordinates are physical pixels.

use std::path::PathBuf;

use tauri::AppHandle;
use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

use super::monitors;
use super::{cursor, input_kind::InputKind, native_settings, target::WindowTarget, tween};
use crate::glide::core::GlideFrame;
use crate::glide::picker_cards;
use crate::glide::region_rect::{PlacedRegion, RegionGravity};
use crate::glide::session::{self as shared, GlidePlatform, Point, Session};

mod access;
mod effects;

pub(super) use access::{
  accumulate_pointer_travel, active_input, anchor, clear_monitor_commit,
  clear_monitor_commit_if_released, monitor_mode, next_id, pointer_displacement,
  promote_armed_to_mouse, promote_scroll_to_contacts, revealed, set_icon, target, target_hwnd,
};

pub(super) struct WindowsGlide;

/// What a Windows session keeps beyond the shared fields.
#[derive(Default)]
pub(super) struct Extra {
  /// The process icon the monitor picker's cards show, once looked up.
  icon_path: Option<PathBuf>,
  /// A trackpad scroll that armed the monitor picker without travelling yet,
  /// which a mouse may still take over.
  armed: bool,
}

type WindowsSession = Session<WindowsGlide>;

static STATE: std::sync::Mutex<Option<WindowsSession>> = std::sync::Mutex::new(None);

fn point(anchor: Point) -> POINT {
  POINT {
    x: anchor.0.round() as i32,
    y: anchor.1.round() as i32,
  }
}

impl GlidePlatform for WindowsGlide {
  type Window = WindowTarget;
  type Input = InputKind;
  type Monitors = monitors::Selection;
  type Extra = Extra;
  type State = Option<WindowsSession>;

  fn with_state<R>(f: impl FnOnce(&mut Self::State) -> R) -> Option<R> {
    STATE.lock().ok().map(|mut state| f(&mut state))
  }

  fn slot(state: &mut Self::State) -> &mut Option<WindowsSession> {
    state
  }

  fn blocks_begin(_app: &AppHandle) -> bool {
    let settings = native_settings::snapshot();
    !settings.enabled
      || super::navigation::cancelled()
      || super::deliberate::blocks_sessions()
      || picker_cards::is_dismissing()
      || crate::shortcuts::is_capturing()
      || native_settings::is_down(settings.spaces_modifier)
      || super::spaces::active_input().is_some()
      || super::spaces::is_closing()
  }

  fn target_at(app: &AppHandle, anchor: Point) -> Option<(WindowTarget, GlideFrame, Option<u32>)> {
    let (target, pid) = WindowTarget::at(app, point(anchor))?;
    let frame = target.frame().ok()?;
    Some((target, frame, pid))
  }

  fn monitor_request() -> Option<bool> {
    let settings = native_settings::snapshot();
    crate::glide::core::trace::input(
      "windows-session",
      format!(
        "begin mouse={} monitors={} spaces={}",
        native_settings::is_down(settings.mouse_modifier),
        native_settings::is_down(settings.monitors_modifier),
        native_settings::is_down(settings.spaces_modifier)
      ),
    );
    if !native_settings::is_down(settings.monitors_modifier) {
      return Some(false);
    }
    // A monitor commit made with the trackpad holds until the modifier lets
    // go, so the same press cannot open a second picker.
    (!access::monitor_commit_latched()).then_some(true)
  }

  fn capture_monitors(app: &AppHandle, anchor: Point) -> Option<monitors::Selection> {
    monitors::capture(app, point(anchor))
  }

  fn work_area(_app: &AppHandle, _anchor: Point, target: &WindowTarget) -> Option<GlideFrame> {
    Some(target.work_area())
  }

  /// The pointer is never held on Windows: the system cursor is hidden while
  /// it moves freely.
  fn hold_pointer(_anchor: Point) -> Result<(), String> {
    Ok(())
  }

  fn present(app: &AppHandle, id: u64, anchor: Point) -> Result<(), String> {
    let anchor = point(anchor);
    crate::glide::begin_physical(app, id, anchor.x, anchor.y)
  }

  fn session_began(_state: &mut Self::State) {}

  fn on_ready(_app: &AppHandle) {}

  fn on_sample(extra: &mut Extra, moved: bool) {
    if moved {
      extra.armed = false;
    }
  }

  fn monitor_effects(app: &AppHandle, id: u64, step: Option<(i8, i8)>, ready: bool) -> bool {
    if let Some(step) = step {
      effects::update_monitor(app, id, Some(step), false);
      return true;
    }
    if !monitor_mode() {
      return false;
    }
    if ready {
      effects::update_monitor(app, id, None, true);
    }
    true
  }

  fn hide_pointer() {
    cursor::hide_cursor();
  }

  fn show_preview(app: &AppHandle, _id: u64, input: InputKind) -> Result<(), String> {
    crate::app_windows::show_glide_island(app, input == InputKind::Mouse)
      .map_err(|error| error.to_string())
  }

  fn region_frame(session: &WindowsSession, region: &PlacedRegion) -> (GlideFrame, RegionGravity) {
    let destination = session
      .target
      .destination(region, native_settings::snapshot().window_gap);
    (destination.frame, destination.gravity)
  }

  fn session_taken(_state: &mut Self::State, session: &WindowsSession, cancelled: bool) {
    if !cancelled
      && session.monitors.is_some()
      && session.input == InputKind::TrackpadContacts
      && native_settings::is_down(native_settings::snapshot().monitors_modifier)
    {
      access::latch_monitor_commit();
    }
  }

  fn end_monitors(app: &AppHandle, session: &mut WindowsSession, cancelled: bool) -> bool {
    let Some(selection) = session.monitors.as_ref() else {
      return false;
    };
    if !cancelled {
      if let Some(frame) = monitors::commit_frame(app, selection, session.original_frame) {
        tween::animate_to(session.target, frame, None);
        session.moved = true;
      }
    }
    picker_cards::dismiss(app, cancelled);
    true
  }

  fn release_pointer(_anchor: Point, show: bool) {
    if show {
      cursor::show_cursor();
    }
  }

  fn land_pointer(point: Point) {
    Self::carry_pointer(point);
    cursor::show_cursor();
  }

  fn carry_pointer(landing: Point) {
    let landing = point(landing);
    screenwide_capture::cursor::set_cursor_visibility(false, None);
    let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::SetCursorPos(landing.x, landing.y) };
    screenwide_capture::cursor::set_cursor_visibility(
      true,
      Some((f64::from(landing.x), f64::from(landing.y))),
    );
  }

  fn after_fade(_app: &AppHandle, _id: u64, completion: crate::glide::BusyLease) {
    picker_cards::after_dismissed(Box::new(move || drop(completion)));
  }
}

pub(super) fn begin(app: &AppHandle, input: InputKind, anchor: POINT) -> bool {
  shared::begin::<WindowsGlide>(app, input, (f64::from(anchor.x), f64::from(anchor.y)))
}

pub(super) fn arm_monitor(app: &AppHandle) -> bool {
  let Some(id) = shared::with_session::<WindowsGlide, _>(|session| {
    session.extra.armed = true;
    session.id
  }) else {
    return false;
  };
  effects::update_monitor(app, id, None, true);
  true
}

pub(super) fn update(app: &AppHandle, delta_x: f64, delta_y: f64, thirds: bool) {
  shared::update::<WindowsGlide>(app, delta_x, delta_y, thirds);
}

pub(super) fn set_thirds(app: &AppHandle, thirds: bool) {
  shared::set_thirds::<WindowsGlide>(app, thirds);
}

pub(super) fn tick(app: &AppHandle) {
  shared::settle::<WindowsGlide>(app);
}

pub(in crate::glide::windows) fn end(app: &AppHandle, cancelled: bool) {
  crate::glide::core::trace::input(
    "windows-session",
    format!(
      "end cancelled={cancelled} active={:?} monitor={}",
      active_input(),
      monitor_mode()
    ),
  );
  shared::end::<WindowsGlide>(app, cancelled, native_settings::snapshot().cursor_follows);
}

/// The pointer as the system reports it, in physical pixels.
fn cursor_position() -> Option<POINT> {
  let mut pointer = POINT::default();
  unsafe { GetCursorPos(&mut pointer) }.ok()?;
  Some(pointer)
}
