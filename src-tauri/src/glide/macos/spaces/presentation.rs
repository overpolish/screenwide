// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  preview_windows::{self, Preview},
  SESSION,
};
use crate::glide::{core::desktops::Snapshot, platform::cursor};
use std::path::PathBuf;
use tauri::AppHandle;

pub(super) fn publish(
  app: &AppHandle,
  id: u64,
  snapshot: &Snapshot,
  destination: Option<&str>,
  phase: &'static str,
) {
  let group = snapshot.groups.iter().find(|group| {
    group
      .desktops
      .iter()
      .any(|desktop| snapshot.membership.contains(&desktop.id))
  });
  let Some(group) = group else {
    return;
  };
  let desktops: Vec<_> = group
    .desktops
    .iter()
    .filter(|desktop| desktop.regular)
    .collect();
  if desktops.is_empty() {
    return;
  }
  let context = SESSION.lock().ok().and_then(|mut state| {
    let session = state
      .as_mut()
      .filter(|session| session.id == id && !session.ending)?;
    if !session.revealed {
      if let Err(error) = cursor::hide_cursor() {
        eprintln!("{error}");
        return None;
      }
      session.revealed = true;
    }
    session.preview_phase = phase;
    let origin = session
      .snapshot
      .as_ref()
      .and_then(|snapshot| snapshot.membership.first())
      .cloned();
    Some((session.anchor, session.icon_path.clone(), origin))
  });
  let Some((anchor, icon_path, origin)) = context else {
    return;
  };
  let active = destination.unwrap_or(&group.current);
  let previews = desktops
    .iter()
    .enumerate()
    .map(|(index, desktop)| Preview {
      session_id: id,
      desktop: desktop.id.clone(),
      selected: desktop.id == active,
      origin: origin.as_deref() == Some(desktop.id.as_str()),
      index,
      count: desktops.len(),
      phase,
      icon_path: icon_path.clone(),
    })
    .collect();
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    if !SESSION.lock().is_ok_and(|state| {
      state
        .as_ref()
        .is_some_and(|session| session.id == id && !session.ending)
    }) {
      return;
    }
    if let Err(error) = preview_windows::show(&main, previews, anchor.x, anchor.y) {
      eprintln!("Could not show Glide Space windows: {error}");
      // Close asynchronously: session cleanup also schedules native UI work.
      super::lifecycle::request_end(&main, true);
    }
  });
}

pub(super) fn set_icon(app: &AppHandle, id: u64, path: Option<PathBuf>) {
  let context = SESSION.lock().ok().and_then(|mut state| {
    let session = state
      .as_mut()
      .filter(|session| session.id == id && !session.ending)?;
    session.icon_path = path;
    Some((
      session.snapshot.clone()?,
      session.selected.clone(),
      session.preview_phase,
    ))
  });
  if let Some((snapshot, selected, phase)) = context {
    publish(app, id, &snapshot, selected.as_deref(), phase);
  }
}
