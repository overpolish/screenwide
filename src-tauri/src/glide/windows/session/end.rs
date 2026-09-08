// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::glide::platform) fn end(app: &AppHandle, cancelled: bool) {
  let completion = crate::glide::core::activity::BusyLease::acquire();
  crate::glide::core::trace::input(
    "windows-session",
    format!(
      "end cancelled={cancelled} active={:?} monitor={}",
      active_input(),
      monitor_mode()
    ),
  );
  if !cancelled {
    let result = STATE.lock().ok().and_then(|mut state| {
      let session = state.as_mut()?;
      let timestamp = session.runtime_clock.elapsed().as_secs_f64() * 1_000.0;
      Some((session.id, session.runtime.finish_opening(timestamp)))
    });
    apply(app, result);
  }
  let session = STATE.lock().ok().and_then(|mut state| state.take());
  let Some(mut session) = session else {
    return;
  };
  if !cancelled
    && session.monitors.is_some()
    && session.input == InputKind::TrackpadContacts
    && native_settings::is_down(native_settings::snapshot().monitors_modifier)
  {
    access::latch_monitor_commit();
  }
  if !cancelled {
    if let Some(selection) = session.monitors.as_ref() {
      if selection.selected != selection.source {
        if let Some(frame) = monitors::commit_frame(app, selection, session.original_frame) {
          tween::animate_to(session.target, frame, None);
          session.moved = true;
        }
      }
      preview_windows::hide(app);
    }
  } else if session.monitors.is_some() {
    preview_windows::hide(app);
  }
  let minimize = session.runtime.should_minimize(cancelled);
  if cancelled && session.moved {
    session.target.restore();
  }
  if minimize {
    session.target.minimize();
  }
  if session.monitors.is_some() && !session.moved {
    let _ = unsafe {
      windows::Win32::UI::WindowsAndMessaging::SetCursorPos(session.anchor.x, session.anchor.y)
    };
  }
  if cancelled || !session.revealed {
    if session.revealed {
      cursor::show_cursor();
    }
    finish(
      app,
      f64::from(session.anchor.x),
      f64::from(session.anchor.y),
      cancelled,
    );
    return;
  }

  // A committed preview stays visible through the final tween and fit
  // correction. Its cursor lease and the activity guard last through the
  // native fade, so a new gesture cannot race the old window or cursor.
  let anchor = session.anchor;
  let target = session.target;
  let cursor_follows = native_settings::snapshot().cursor_follows
    && !session.runtime.commits_terminal_action(cancelled)
    && session.moved;
  let app = app.clone();
  tween::after_current(Box::new(move || {
    let restore_completion = completion.clone();
    let restore = Box::new(move || {
      if cursor_follows {
        if let Some(landing) = target.landing(anchor) {
          tween::land_cursor(landing, restore_completion.clone());
        }
      }
      cursor::show_cursor();
    });
    let faded_completion = Box::new(move || {
      let _completion = completion;
    });
    finish_with_fade(
      &app,
      f64::from(anchor.x),
      f64::from(anchor.y),
      restore,
      faded_completion,
    );
  }));
}
