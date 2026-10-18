// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS supplies monitor geometry and applies the shared destination policy.
use super::{Session, SharedState, STATE};
use crate::glide::{
  core::{
    monitors::{self, Monitor},
    GlideFrame,
  },
  platform::cursor,
  preview_cards::{self, Preview},
};
use core_graphics::geometry::CGPoint;
use std::path::PathBuf;
use tauri::AppHandle;

pub(crate) struct Selection {
  displays: Vec<Monitor>,
  source: usize,
  selected: usize,
  pub active: bool,
  pub armed: bool,
  icon_path: Option<PathBuf>,
  preview_phase: &'static str,
}

fn snapshot(app: &AppHandle) -> Vec<Monitor> {
  let mut displays = Vec::new();
  let topology = match crate::monitor_topology::snapshot(app) {
    Ok(topology) => topology,
    Err(error) => {
      eprintln!("Could not read Glide monitor topology: {error}");
      return displays;
    }
  };
  for display in topology {
    let monitor = display.native;
    let scale = monitor.scale_factor();
    let pos = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);
    let area = monitor.work_area();
    let work_pos = area.position.to_logical::<f64>(scale);
    let work_size = area.size.to_logical::<f64>(scale);
    let bounds = GlideFrame {
      x: pos.x,
      y: pos.y,
      width: size.width,
      height: size.height,
    };
    if displays.iter().any(|item: &Monitor| item.bounds == bounds) {
      continue;
    }
    displays.push(Monitor {
      id: display.id.to_string(),
      layout: GlideFrame {
        x: display.layout_position.0,
        y: display.layout_position.1,
        width: display.layout_size.0,
        height: display.layout_size.1,
      },
      bounds,
      work: GlideFrame {
        x: work_pos.x,
        y: work_pos.y,
        width: work_size.width,
        height: work_size.height,
      },
    });
  }
  displays
}

pub(super) fn capture(app: &AppHandle, point: CGPoint) -> Option<Selection> {
  let displays = snapshot(app);
  if displays.is_empty() {
    return None;
  }
  let source = displays.iter().position(|monitor| {
    let r = monitor.bounds;
    point.x >= r.x && point.x < r.x + r.width && point.y >= r.y && point.y < r.y + r.height
  })?;
  Some(Selection {
    displays,
    source,
    selected: source,
    active: false,
    armed: false,
    icon_path: None,
    preview_phase: "idle",
  })
}

/// Returns true while monitor selection owns the preview and blocks resizing.
pub(super) fn update(app: &AppHandle, id: u64, step: Option<(i8, i8)>, ready: bool) -> bool {
  let Some(state) = STATE.get() else {
    return false;
  };
  let active = state.lock().is_ok_and(|mut state| {
    let Some(session) = state.session.as_mut().filter(|session| session.id == id) else {
      return false;
    };
    let Some(selection) = session.monitors.as_mut() else {
      return false;
    };
    if ready {
      selection.active = true;
      selection.preview_phase = "ready";
    }
    if let Some(direction) = step {
      selection.active = true;
      selection.armed = false;
      selection.preview_phase = "settling";
      if let Some(next) = monitors::neighbour(&selection.displays, selection.selected, direction) {
        selection.selected = next;
      }
    }
    selection.active
  });
  if active && (step.is_some() || ready) {
    publish(app, id);
  }
  active
}

mod arming;
use arming::publish;
pub(super) use arming::{arm_preview, claim_armed, current_id, is_armed};

/// Keeps the icon for the cards the picker shows from now on; the cards up
/// already take it from `preview_cards::set_icon`.
pub(in crate::glide::macos) fn set_icon(id: u64, path: Option<PathBuf>) {
  let Some(state) = STATE.get() else {
    return;
  };
  if let Ok(mut state) = state.lock() {
    if let Some(selection) = state
      .session
      .as_mut()
      .filter(|session| session.id == id)
      .and_then(|session| session.monitors.as_mut())
    {
      selection.icon_path = path;
    }
  }
}

/// Revalidate the selected physical display on release before applying a frame.
pub(super) fn commit(app: &AppHandle, session: &mut Session) {
  crate::glide::core::trace::input(
    "mac-monitor",
    format!(
      "commit id={} active={}",
      session.id,
      session
        .monitors
        .as_ref()
        .is_some_and(|selection| selection.active)
    ),
  );
  let Some(selection) = session
    .monitors
    .as_ref()
    .filter(|selection| selection.active)
  else {
    return;
  };
  if selection.selected == selection.source {
    return;
  }
  let selected = &selection.displays[selection.selected];
  let Some(destination) = snapshot(app)
    .into_iter()
    .find(|monitor| monitor.id == selected.id)
  else {
    eprintln!("Glide monitor move: selected display is no longer available");
    return;
  };
  let frame = monitors::carry_frame(
    session.original_frame,
    selection.displays[selection.source].work,
    destination.work,
  );
  session.work = destination.work;
  session.moved = true;
  session.returned_to_origin = false;
  crate::glide::core::tween::animate_to(session.target.duplicate(), frame, None);
}
