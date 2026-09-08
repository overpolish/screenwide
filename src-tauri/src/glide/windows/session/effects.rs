// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn session_mut(session: &mut Option<Session>, session_id: u64) -> Result<&mut Session, String> {
  session
    .as_mut()
    .filter(|session| session.id == session_id)
    .ok_or_else(|| "The Glide session has already ended".to_owned())
}

pub(crate) fn reveal(app: &AppHandle, session_id: u64) -> Result<(), String> {
  let blocks_hover = {
    let mut state = STATE
      .lock()
      .map_err(|_| "The Glide session state is unavailable".to_owned())?;
    let session = session_mut(&mut state, session_id)?;
    if session.revealed {
      None
    } else {
      session.revealed = true;
      Some(session.input == InputKind::Mouse)
    }
  };
  if let Some(blocks_hover) = blocks_hover {
    cursor::hide_cursor();
    crate::windows::show_glide_preview(app, blocks_hover).map_err(|error| error.to_string())?;
  }
  Ok(())
}

pub(crate) fn region_moved(
  app: &AppHandle,
  session_id: u64,
  region: &PlacedRegion,
) -> Result<(), String> {
  let target = {
    let mut state = STATE
      .lock()
      .map_err(|_| "The Glide session state is unavailable".to_owned())?;
    let session = session_mut(&mut state, session_id)?;
    session.moved = true;
    session.target
  };
  let destination = target.destination(region, native_settings::snapshot().window_gap);
  tween::animate_to(
    target,
    destination.frame,
    Some(FitContext {
      app: app.clone(),
      session_id,
      gravity: destination.gravity,
      work: destination.work,
    }),
  );
  Ok(())
}

pub(crate) fn apply(app: &AppHandle, result: Option<(u64, GlideEffects)>) {
  let Some((session_id, effects)) = result else {
    return;
  };
  events::detection(app, effects.detection);
  if let Some(step) = effects.monitor_step {
    update_monitor(app, session_id, Some(step), false);
    return;
  }
  if STATE.lock().ok().is_some_and(|state| {
    state
      .as_ref()
      .is_some_and(|session| session.monitors.is_some())
  }) {
    if effects.ready {
      update_monitor(app, session_id, None, true);
    }
    return;
  }
  if effects.reveal {
    if let Err(error) = reveal(app, session_id) {
      eprintln!("Could not reveal Glide: {error}");
    }
  }
  if let Some(region) = effects.move_to {
    if let Err(error) = region_moved(app, session_id, &region) {
      eprintln!("Could not move the Glide window: {error}");
    }
  }
}

pub(crate) fn update_monitor(app: &AppHandle, id: u64, direction: Option<(i8, i8)>, ready: bool) {
  crate::glide::core::trace::input(
    "windows-monitor",
    format!("update id={id} direction={direction:?} ready={ready}"),
  );
  let Some(Some((anchor, offsets, cards, hide))) = STATE.lock().ok().map(|mut state| {
    let session = state.as_mut()?.id.eq(&id).then_some(state.as_mut()?)?;
    let selection = session.monitors.as_mut()?;
    if let Some(direction) = direction {
      if let Some(next) =
        crate::glide::core::monitors::neighbour(&selection.displays, selection.selected, direction)
      {
        selection.selected = next;
      }
    }
    let hide = !session.revealed;
    session.revealed = true;
    let offsets = crate::glide::core::monitors::preview_offsets(&selection.displays);
    let cards = selection
      .displays
      .iter()
      .enumerate()
      .map(|(index, monitor)| preview_windows::Preview {
        session_id: id,
        desktop: monitor.id.clone(),
        selected: index == selection.selected,
        origin: index == selection.source,
        index,
        count: selection.displays.len(),
        phase: if ready { "ready" } else { "settling" },
        icon_path: session.icon_path.clone(),
      })
      .collect();
    Some((session.anchor, offsets, cards, hide))
  }) else {
    return;
  };
  if hide {
    cursor::hide_cursor();
  }
  preview_windows::show_arranged(
    app,
    cards,
    f64::from(anchor.x),
    f64::from(anchor.y),
    &offsets,
  );
  // Reconcile after show_arranged installs CURRENT. Keep the session lock
  // while reading and applying the icon so an icon lookup cannot interleave
  // between the read and setter and then be overwritten by stale None.
  if let Ok(state) = STATE.lock() {
    if let Some(session) = state.as_ref().filter(|session| session.id == id) {
      preview_windows::set_icon(app, id, session.icon_path.clone());
    }
  }
}
