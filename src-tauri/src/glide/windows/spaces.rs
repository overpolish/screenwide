// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::{input_kind::InputKind, preview_windows, target::WindowTarget, virtual_desktops};
use crate::glide::core::{
  desktop_gesture::DesktopGesture,
  desktop_selection,
  desktops::{self, Snapshot},
};
use std::{
  path::PathBuf,
  sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
  },
  time::Instant,
};
use tauri::AppHandle;
use windows::Win32::Foundation::POINT;
#[path = "spaces/presentation.rs"]
mod presentation;
struct Session {
  id: u64,
  input: InputKind,
  hwnd: isize,
  anchor: POINT,
  snapshot: Snapshot,
  selected: Option<String>,
  gesture: DesktopGesture,
  clock: Instant,
  revealed: bool,
  icon_path: Option<PathBuf>,
  armed: bool,
}
static STATE: Mutex<Option<Session>> = Mutex::new(None);
static CLOSING: AtomicBool = AtomicBool::new(false);
static COMMITTED_UNTIL_RELEASE: AtomicBool = AtomicBool::new(false);
pub(super) fn supported() -> bool {
  virtual_desktops::supported()
}
pub(super) fn begin(
  app: &AppHandle,
  target: WindowTarget,
  anchor: POINT,
  input: InputKind,
) -> bool {
  if super::navigation::cancelled() {
    return false;
  }
  if crate::glide::core::activity::BusyLease::is_busy() {
    return false;
  }
  if COMMITTED_UNTIL_RELEASE.load(Ordering::Acquire) {
    return false;
  }
  let s = super::native_settings::snapshot();
  crate::glide::core::trace::input(
    "windows-spaces",
    format!(
      "begin input={input:?} spaces={} closing={}",
      super::native_settings::is_down(s.spaces_modifier),
      CLOSING.load(Ordering::Acquire)
    ),
  );
  if super::session::active_input().is_some() || crate::shortcuts::is_capturing() {
    return false;
  }
  if !supported()
    || !s.enabled
    || !super::native_settings::is_down(s.spaces_modifier)
    || crate::capture_overlays::blocks_glide(app)
    || CLOSING.load(Ordering::Acquire)
  {
    return false;
  }
  let a = virtual_desktops::Adapter::new(target.native_hwnd());
  let Ok(snapshot) = desktops::DesktopAdapter::snapshot(&a) else {
    return false;
  };
  let regular: Vec<&str> = snapshot
    .groups
    .iter()
    .flat_map(|group| group.desktops.iter())
    .filter(|desktop| desktop.regular)
    .map(|desktop| desktop.id.as_str())
    .collect();
  if snapshot.membership.len() != 1
    || !snapshot.groups.iter().any(|group| {
      !group.transitioning
        && group.current == snapshot.membership[0]
        && group
          .desktops
          .iter()
          .any(|desktop| desktop.id == group.current && desktop.regular)
    })
    || !regular.contains(&snapshot.membership[0].as_str())
    || regular.len() < 2
  {
    return false;
  }
  let Ok(mut st) = STATE.lock() else {
    return false;
  };
  if st.is_some() {
    return false;
  }
  let id = super::session::next_id();
  let pid = super::titlebar::process_id(target.native_hwnd());
  *st = Some(Session {
    id,
    input,
    hwnd: target.native_hwnd().0 as isize,
    anchor,
    snapshot,
    selected: None,
    gesture: DesktopGesture::default(),
    clock: Instant::now(),
    revealed: false,
    icon_path: None,
    armed: false,
  });
  crate::glide::icon::spawn_icon_lookup(app, id, pid);
  true
}

pub(super) fn arm(app: &AppHandle, target: WindowTarget, anchor: POINT) -> bool {
  if !begin(app, target, anchor, InputKind::TrackpadScroll) {
    return false;
  }
  if let Ok(mut state) = STATE.lock() {
    if let Some(session) = state.as_mut() {
      session.armed = true;
    }
  }
  if let Ok(mut state) = STATE.lock() {
    if let Some(session) = state.as_mut() {
      presentation::publish(app, session, "idle");
    }
  }
  true
}

pub(super) fn promote_armed_to_mouse() -> bool {
  STATE.lock().is_ok_and(|mut state| {
    let Some(session) = state.as_mut() else {
      return false;
    };
    if session.input == InputKind::TrackpadScroll
      && crate::glide::core::armed::claim(
        &mut session.armed,
        crate::glide::core::armed::Owner::Mouse,
      )
      .is_some()
    {
      session.input = InputKind::Mouse;
      true
    } else {
      false
    }
  })
}
pub(super) fn handle_event(app: &AppHandle, dx: f64, dy: f64) -> bool {
  let Ok(mut st) = STATE.lock() else {
    return false;
  };
  let Some(s) = st.as_mut() else { return false };
  s.armed = false;
  let t = s.clock.elapsed().as_secs_f64() * 1000.;
  if let Some(d) = s.gesture.update(dx, dy, t) {
    if let Ok(id) = desktop_selection::step(&s.snapshot, s.selected.as_deref(), d) {
      s.selected = Some(id);
    }
    s.gesture.completed(t);
    presentation::publish(app, s, "settling");
  }
  true
}

pub(super) fn promote_scroll_to_contacts() -> bool {
  STATE.lock().is_ok_and(|mut state| {
    let Some(session) = state.as_mut() else {
      return false;
    };
    if session.input != InputKind::TrackpadScroll {
      return false;
    }
    session.armed = false;
    session.input = InputKind::TrackpadContacts;
    true
  })
}
pub(super) fn poll(app: &AppHandle) {
  if let Ok(mut st) = STATE.lock() {
    if let Some(s) = st.as_mut() {
      let t = s.clock.elapsed().as_secs_f64() * 1000.;
      if s.gesture.settle(t) {
        presentation::publish(app, s, "ready");
      }
    }
  }
}
pub(super) fn end(app: &AppHandle, cancelled: bool) {
  let Ok(mut st) = STATE.lock() else { return };
  let Some(s) = st.take() else { return };
  if !cancelled
    && s.input == InputKind::TrackpadContacts
    && super::native_settings::is_down(super::native_settings::snapshot().spaces_modifier)
  {
    COMMITTED_UNTIL_RELEASE.store(true, Ordering::Release);
  }
  preview_windows::hide(app);
  if s.revealed {
    if cancelled
      || s
        .selected
        .as_ref()
        .is_none_or(|dest| dest == &s.snapshot.membership[0])
    {
      let _ =
        unsafe { windows::Win32::UI::WindowsAndMessaging::SetCursorPos(s.anchor.x, s.anchor.y) };
    }
    super::cursor::show_cursor()
  }
  if cancelled {
    return;
  }
  let Some(dest) = s.selected else { return };
  let source = s.snapshot.membership[0].clone();
  if dest == source {
    return;
  }
  CLOSING.store(true, Ordering::Release);
  let completion = crate::glide::core::activity::BusyLease::acquire();
  let raw = s.hwnd;
  let spawned = std::thread::Builder::new()
    .name("glide-spaces-carry".into())
    .spawn(move || {
      let _completion = completion;
      let a = virtual_desktops::Adapter::new(windows::Win32::Foundation::HWND(
        raw as *mut std::ffi::c_void,
      ));
      if let Err(e) = desktops::move_to_and_follow(&a, "windows-global", &source, &dest) {
        eprintln!("Glide Spaces carry failed: {e:?}")
      }
      CLOSING.store(false, Ordering::Release);
    });
  if let Err(error) = spawned {
    eprintln!("Could not start Glide Spaces carry: {error}");
    CLOSING.store(false, Ordering::Release);
  }
}
pub(super) fn cancel(app: &AppHandle) {
  crate::glide::core::trace::input("windows-spaces", "cancel-requested");
  end(app, true)
}
pub(super) fn active_input() -> Option<InputKind> {
  STATE.lock().ok().and_then(|s| s.as_ref().map(|s| s.input))
}
pub(super) fn is_closing() -> bool {
  CLOSING.load(Ordering::Acquire)
}

pub(super) fn clear_committed_if_released() {
  if !super::native_settings::is_down(super::native_settings::snapshot().spaces_modifier) {
    COMMITTED_UNTIL_RELEASE.store(false, Ordering::Release);
  }
}

pub(super) fn clear_committed() {
  COMMITTED_UNTIL_RELEASE.store(false, Ordering::Release);
}
pub(super) fn anchor() -> Option<POINT> {
  STATE.lock().ok().and_then(|s| s.as_ref().map(|s| s.anchor))
}

pub(super) fn set_icon(id: u64, path: Option<PathBuf>) {
  if let Ok(mut state) = STATE.lock() {
    if let Some(session) = state.as_mut().filter(|session| session.id == id) {
      session.icon_path = path;
    }
  }
}
