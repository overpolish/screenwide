// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::input_kind::InputKind;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MotionRoute {
  Monitor,
  Ignore,
  Dismiss,
  Normal,
}

fn mouse_motion_route(active: Option<InputKind>, monitor: bool) -> MotionRoute {
  match (active, monitor) {
    (Some(input), true) if input.is_trackpad() => MotionRoute::Ignore,
    (Some(input), _) if input.is_trackpad() => MotionRoute::Dismiss,
    (Some(InputKind::Mouse), true) => MotionRoute::Monitor,
    _ => MotionRoute::Normal,
  }
}

pub(crate) fn handle_mouse_delta(delta_x: i32, delta_y: i32) {
  if session::monitor_mode()
    && session::active_input() == Some(InputKind::TrackpadScroll)
    && !native_trackpad::pointer_episode_active()
    && (delta_x != 0 || delta_y != 0)
  {
    session::promote_armed_to_mouse();
  }
  match mouse_motion_route(session::active_input(), session::monitor_mode()) {
    MotionRoute::Ignore => {
      center::trace("mouse drift ignored for trackpad monitor navigation");
      return;
    }
    MotionRoute::Dismiss => {
      let distance = f64::from(delta_x.unsigned_abs()) + f64::from(delta_y.unsigned_abs());
      if session::accumulate_pointer_travel(distance) >= POINTER_DISMISS_DISTANCE {
        if let Some(app) = APP.get() {
          session::end(app, false);
        }
      }
      return;
    }
    MotionRoute::Monitor | MotionRoute::Normal => {}
  }
  let settings = native_settings::snapshot();
  if spaces::active_input().is_some() || native_settings::is_down(settings.spaces_modifier) {
    let mut active = spaces::active_input();
    if active == Some(InputKind::TrackpadScroll)
      && !native_trackpad::pointer_episode_active()
      && spaces::promote_armed_to_mouse()
    {
      active = spaces::active_input();
    }
    if !navigation::owns(active, InputKind::Mouse) {
      center::trace(&format!("mouse navigation rejected active={active:?}"));
      return;
    }
    let Some(app) = APP.get() else {
      return;
    };
    if spaces::active_input().is_none() {
      let point = session::anchor().unwrap_or_else(|| {
        let mut p = windows::Win32::Foundation::POINT::default();
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut p) };
        p
      });
      if let Some((target, _)) = target::WindowTarget::at(app, point) {
        if !spaces::begin(app, target, point, InputKind::Mouse) {
          return;
        }
      }
    }
    spaces::handle_event(app, f64::from(delta_x), f64::from(delta_y));
    if let Some(anchor) = spaces::anchor() {
      let _ = unsafe { SetCursorPos(anchor.x, anchor.y) };
    }
    return;
  }
  if (matches!(
    mouse_motion_route(session::active_input(), session::monitor_mode()),
    MotionRoute::Monitor
  ) && !native_trackpad::pointer_episode_active())
    || (native_settings::is_down(settings.monitors_modifier)
      && session::active_input().is_none()
      && !native_trackpad::pointer_episode_active())
  {
    if session::active_input().is_none() && !begin_session(InputKind::Mouse) {
      return;
    }
    if let Some(app) = APP.get() {
      session::update(app, f64::from(delta_x), f64::from(delta_y), false);
    }
    if let Some(anchor) = session::anchor() {
      let _ = unsafe { SetCursorPos(anchor.x, anchor.y) };
    }
    return;
  }
  let modifier_down = native_settings::is_down(settings.mouse_modifier);
  if native_trackpad::blocks_mouse_glide(modifier_down) {
    finish_current_session(true);
    return;
  }
  if session::active_input() == Some(InputKind::Mouse) && !modifier_down {
    finish_current_session(false);
    return;
  }
  if session::active_input().is_none() {
    if !modifier_down {
      return;
    }
    if !begin_session(InputKind::Mouse) {
      return;
    }
  }
  if session::active_input() != Some(InputKind::Mouse) {
    return;
  }
  if let Some(anchor) = session::anchor() {
    let _ = unsafe { SetCursorPos(anchor.x, anchor.y) };
  }
  if let Some(app) = APP.get() {
    session::update(
      app,
      f64::from(delta_x),
      f64::from(delta_y),
      native_settings::is_down(settings.thirds_modifier),
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mouse_monitor_accepts_motion() {
    assert_eq!(
      mouse_motion_route(Some(InputKind::Mouse), true),
      MotionRoute::Monitor
    );
  }

  #[test]
  fn trackpad_monitor_ignores_mouse_drift() {
    assert_eq!(
      mouse_motion_route(Some(InputKind::TrackpadContacts), true),
      MotionRoute::Ignore
    );
    assert_eq!(
      mouse_motion_route(Some(InputKind::TrackpadScroll), true),
      MotionRoute::Ignore
    );
  }

  #[test]
  fn ordinary_trackpad_motion_dismisses() {
    assert_eq!(
      mouse_motion_route(Some(InputKind::TrackpadContacts), false),
      MotionRoute::Dismiss
    );
  }
}
