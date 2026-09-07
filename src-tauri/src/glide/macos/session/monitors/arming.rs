// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::glide::platform::session) fn claim_armed(
  state: &SharedState,
  input: super::super::InputKind,
) -> bool {
  state.lock().is_ok_and(|mut state| {
    let Some(session) = state.session.as_mut() else {
      return false;
    };
    let Some(selection) = session.monitors.as_mut() else {
      return false;
    };
    if !selection.armed {
      return false;
    }
    selection.armed = false;
    session.input = input;
    true
  })
}

pub(in crate::glide::platform::session) fn current_id(state: &SharedState) -> Option<u64> {
  state
    .lock()
    .ok()
    .and_then(|state| state.session.as_ref().map(|session| session.id))
}

pub(in crate::glide::platform::session) fn is_armed(state: &SharedState) -> bool {
  state.lock().is_ok_and(|state| {
    state
      .session
      .as_ref()
      .and_then(|session| session.monitors.as_ref())
      .is_some_and(|selection| selection.armed)
  })
}

pub(in crate::glide::platform::session) fn arm_preview(app: &AppHandle, id: u64) {
  crate::glide::core::trace::input("mac-monitor", format!("arm-preview id={id}"));
  if let Some(state) = STATE.get() {
    if state.lock().is_ok_and(|mut state| {
      let Some(session) = state.session.as_mut().filter(|session| session.id == id) else {
        return false;
      };
      let Some(selection) = session.monitors.as_mut() else {
        return false;
      };
      selection.active = true;
      selection.armed = true;
      true
    }) {
      publish(app, id);
    }
  }
}

pub(super) fn publish(app: &AppHandle, id: u64) {
  let Some(state) = super::STATE.get() else {
    return;
  };
  let ready = state.lock().ok().and_then(|mut state| {
    let session = state.session.as_mut().filter(|session| session.id == id)?;
    if !session
      .monitors
      .as_ref()
      .is_some_and(|selection| selection.active)
    {
      return None;
    }
    if !session.revealed {
      if let Err(error) = cursor::hide_cursor() {
        eprintln!("{error}");
        return None;
      }
      session.revealed = true;
    }
    Some(())
  });
  if ready.is_none() {
    return;
  }
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    let Some((anchor, previews, offsets)) = state.lock().ok().and_then(|state| {
      let session = state.session.as_ref().filter(|session| session.id == id)?;
      let selection = session
        .monitors
        .as_ref()
        .filter(|selection| selection.active)?;
      let previews = selection
        .displays
        .iter()
        .enumerate()
        .map(|(index, monitor)| Preview {
          session_id: id,
          desktop: monitor.id.clone(),
          selected: selection.selected == index,
          origin: selection.source == index,
          index,
          count: selection.displays.len(),
          phase: selection.preview_phase,
          icon_path: selection.icon_path.clone(),
        })
        .collect::<Vec<_>>();
      Some((
        session.anchor,
        previews,
        monitors::preview_offsets(&selection.displays),
      ))
    }) else {
      return;
    };
    let _ = crate::windows::hide_glide_preview(&main);
    if let Err(error) =
      preview_windows::show_arranged(&main, previews, anchor.x, anchor.y, &offsets)
    {
      eprintln!("Could not show Glide monitor previews: {error}");
    }
  });
}
