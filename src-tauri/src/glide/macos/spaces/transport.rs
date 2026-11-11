// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use super::{adapter::MacDesktopAdapter, lifecycle, presentation, Session, SESSION};
use crate::glide::core::{
  desktop_selection,
  desktops::{self, DesktopAdapter},
};
use crate::glide::picker_cards;
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
        // A window on every Space, as the island is, cannot be carried
        // to another one, so it is offered no Spaces to pick from.
        Ok(snapshot) if snapshot.membership.len() != 1 => {
          if SESSION
            .lock()
            .is_ok_and(|state| state.as_ref().is_some_and(|session| session.id == id))
          {
            super::input::cancel_until_release(&worker_app);
          }
        }
        Ok(snapshot) => {
          let preview = SESSION.lock().ok().and_then(|mut state| {
            let session = state.as_mut().filter(|session| session.id == id)?;
            session.snapshot = Some(snapshot.clone());
            let (x, y) = std::mem::take(&mut session.buffered);
            select(session, x, y);
            Some(session.selected.clone())
          });
          if let Some(selected) = preview {
            presentation::publish(&worker_app, id, &snapshot, selected.as_deref());
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

/// Steps the selection by one sample, answering whether the sample made a
/// step, taken or not.
fn select(session: &mut Session, x: f64, y: f64) -> bool {
  let Some(snapshot) = session.snapshot.as_ref() else {
    return false;
  };
  let time = session.clock.elapsed().as_secs_f64() * 1000.0;
  let Some(direction) = session.detector.update(x, y, time) else {
    return false;
  };
  if let Ok(selected) = desktop_selection::step(snapshot, session.selected.as_deref(), direction) {
    session.selected = Some(selected);
  }
  true
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
    if !select(session, x, y) {
      return None;
    }
    Some((
      session.id,
      session.snapshot.clone()?,
      session.selected.clone(),
    ))
  });
  if let Some((id, snapshot, selected)) = preview {
    presentation::publish(app, id, &snapshot, selected.as_deref());
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
  picker_cards::dismiss(app, false);
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
