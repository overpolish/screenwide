// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  adapter::MacDesktopAdapter, lifecycle, presentation, preview_windows, Session, SESSION,
};
use crate::glide::core::{
  desktop_selection,
  desktops::{self, DesktopAdapter},
};
use std::sync::atomic::Ordering;
use tauri::AppHandle;

pub(super) fn inspect(app: &AppHandle, id: u64) {
  let context = SESSION.lock().ok().and_then(|state| {
    let session = state.as_ref().filter(|session| session.id == id)?;
    Some((
      session.target.duplicate(),
      session.anchor,
      session.cancelled.clone(),
    ))
  });
  let Some((target, anchor, cancelled)) = context else {
    return;
  };
  let worker_app = app.clone();
  let result = std::thread::Builder::new()
    .name("glide-space-layout".into())
    .spawn(move || {
      let result =
        MacDesktopAdapter::new(&target, anchor, cancelled).and_then(|adapter| adapter.snapshot());
      match result {
        Ok(snapshot) => {
          let preview = SESSION.lock().ok().and_then(|mut state| {
            let session = state.as_mut().filter(|session| session.id == id)?;
            session.snapshot = Some(snapshot.clone());
            let (x, y) = std::mem::take(&mut session.buffered);
            let phase = select(session, x, y).unwrap_or("idle");
            Some((session.selected.clone(), phase))
          });
          if let Some((selected, phase)) = preview {
            presentation::publish(&worker_app, id, &snapshot, selected.as_deref(), phase);
            commit(&worker_app);
          }
        }
        Err(error) => {
          eprintln!("Glide Spaces: {error:?}");
          if SESSION
            .lock()
            .is_ok_and(|state| state.as_ref().is_some_and(|session| session.id == id))
          {
            lifecycle::request_end(&worker_app, true);
          }
        }
      }
    });
  if let Err(error) = result {
    eprintln!("Could not inspect Spaces: {error}");
    lifecycle::request_end(app, true);
  }
}

fn select(session: &mut Session, x: f64, y: f64) -> Option<&'static str> {
  let snapshot = session.snapshot.as_ref()?;
  let time = session.clock.elapsed().as_secs_f64() * 1000.0;
  let direction = session.detector.update(x, y, time)?;
  // A selection step rearms after stillness; no native carry happens here.
  session.detector.completed(time);
  match desktop_selection::step(snapshot, session.selected.as_deref(), direction) {
    Ok(selected) => {
      session.selected = Some(selected);
      Some("settling")
    }
    Err(_) => Some("blocked"),
  }
}

pub(super) fn update(app: &AppHandle, x: f64, y: f64) {
  let preview = SESSION.lock().ok().and_then(|mut state| {
    let session = state.as_mut()?;
    if session.ending {
      return None;
    }
    session.last_input = std::time::Instant::now();
    if session.snapshot.is_none() {
      session.buffered.0 += x;
      session.buffered.1 += y;
      return None;
    }
    let phase = select(session, x, y)?;
    Some((
      session.id,
      session.snapshot.clone()?,
      session.selected.clone(),
      phase,
    ))
  });
  if let Some((id, snapshot, selected, phase)) = preview {
    presentation::publish(app, id, &snapshot, selected.as_deref(), phase);
  }
}

/// Called on lift/modifier release or wheel idle. Repeated calls are harmless.
pub(super) fn commit(app: &AppHandle) {
  let mut close = false;
  let command = SESSION.lock().ok().and_then(|mut state| {
    let session = state.as_mut()?;
    if !session.ending || session.moving {
      return None;
    }
    if session.cancelled.load(Ordering::Acquire) {
      close = true;
      return None;
    }
    let snapshot = session.snapshot.as_ref()?; // layout may still be loading
    let Some(destination) = session.selected.as_ref() else {
      close = true;
      return None;
    };
    let [source] = snapshot.membership.as_slice() else {
      close = true;
      return None;
    };
    if destination == source {
      close = true;
      return None;
    }
    let Some(group) = snapshot
      .groups
      .iter()
      .find(|group| group.current == *source)
    else {
      close = true;
      return None;
    };
    session.moving = true;
    Some((
      session.id,
      session.target.duplicate(),
      session.anchor,
      session.cancelled.clone(),
      group.id.clone(),
      source.clone(),
      destination.clone(),
    ))
  });
  if close {
    lifecycle::close(app);
  }
  let Some((id, target, anchor, cancelled, group, source, destination)) = command else {
    return;
  };
  // Commit ends preview presentation; the native carry keeps cursor ownership.
  preview_windows::dismiss(app, false);
  let worker_app = app.clone();
  let result = std::thread::Builder::new()
    .name("glide-space-carry".into())
    .spawn(move || {
      let result = MacDesktopAdapter::new(&target, anchor, cancelled.clone()).and_then(|adapter| {
        if cancelled.load(Ordering::Acquire) {
          Err(desktops::MoveError::FollowUnconfirmed)
        } else {
          desktops::move_to_and_follow(&adapter, &group, &source, &destination)
        }
      });
      if let Err(error) = &result {
        eprintln!("Glide Spaces: {error:?}");
      }
      complete(&worker_app, id);
    });
  if let Err(error) = result {
    eprintln!("Could not start Spaces carry: {error}");
    complete(app, id);
  }
}

fn complete(app: &AppHandle, id: u64) {
  let accepted = SESSION
    .lock()
    .is_ok_and(|state| state.as_ref().is_some_and(|session| session.id == id));
  if !accepted {
    return;
  }
  lifecycle::close(app);
}
