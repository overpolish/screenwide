// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::SESSION;
use crate::glide::{
  core::desktops::Snapshot,
  platform::cursor,
  preview_cards::{self, Anchor, Preview},
};
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
    .collect::<Vec<_>>();
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    if !SESSION.lock().is_ok_and(|state| {
      state
        .as_ref()
        .is_some_and(|session| session.id == id && !session.ending)
    }) {
      return;
    }
    let offsets: Vec<_> = (0..previews.len())
      .map(|index| (index as f64 * 56.0, 0.0))
      .collect();
    preview_cards::show_arranged(
      &main,
      previews,
      Anchor::Logical(anchor.x, anchor.y),
      &offsets,
    );
  });
}

/// Keeps the icon for the cards Spaces shows from now on; the cards up
/// already take it from `preview_cards::set_icon`.
pub(super) fn set_icon(id: u64, path: Option<PathBuf>) {
  if let Ok(mut state) = SESSION.lock() {
    if let Some(session) = state
      .as_mut()
      .filter(|session| session.id == id && !session.ending)
    {
      session.icon_path = path;
    }
  }
}
