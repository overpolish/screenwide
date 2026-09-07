// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS supplies monitor geometry and applies the shared destination policy.
use super::{Session, STATE};
use crate::glide::{
  core::{
    monitors::{self, Monitor},
    GlideFrame,
  },
  platform::{
    cursor,
    spaces::preview_windows::{self, Preview},
  },
};
use core_graphics::geometry::CGPoint;
use std::path::PathBuf;
use tauri::AppHandle;

pub(super) struct Selection {
  displays: Vec<Monitor>,
  source: usize,
  selected: usize,
  pub active: bool,
  icon_path: Option<PathBuf>,
  landing_frame: Option<cidre::cg::Rect>,
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
  if displays.len() < 2 {
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
    icon_path: None,
    landing_frame: None,
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
    if let Some(direction) = step {
      selection.active = true;
      if let Some(next) = monitors::neighbour(&selection.displays, selection.selected, direction) {
        selection.selected = next;
      }
    }
    selection.active
  });
  if active && (step.is_some() || ready) {
    publish(app, id, if ready { "ready" } else { "settling" });
  }
  active
}

fn publish(app: &AppHandle, id: u64, phase: &'static str) {
  let Some(state) = STATE.get() else {
    return;
  };
  let context = state.lock().ok().and_then(|mut state| {
    let session = state.session.as_mut().filter(|session| session.id == id)?;
    let selection = session
      .monitors
      .as_ref()
      .filter(|selection| selection.active)?;
    if !session.revealed {
      if let Err(error) = cursor::hide_cursor() {
        eprintln!("{error}");
        return None;
      }
      session.revealed = true;
    }
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
        phase,
        icon_path: selection.icon_path.clone(),
      })
      .collect();
    Some((
      session.anchor,
      previews,
      monitors::preview_offsets(&selection.displays),
    ))
  });
  let Some((anchor, previews, offsets)) = context else {
    return;
  };
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    if !state.lock().is_ok_and(|state| {
      state
        .session
        .as_ref()
        .is_some_and(|session| session.id == id)
    }) {
      return;
    }
    let _ = crate::windows::hide_glide_preview(&main);
    if let Err(error) =
      preview_windows::show_arranged(&main, previews, anchor.x, anchor.y, &offsets)
    {
      eprintln!("Could not show Glide monitor previews: {error}");
    }
  });
}

pub(in crate::glide::platform) fn set_icon(app: &AppHandle, id: u64, path: Option<PathBuf>) {
  let Some(state) = STATE.get() else {
    return;
  };
  let active = state.lock().is_ok_and(|mut state| {
    let Some(selection) = state
      .session
      .as_mut()
      .filter(|session| session.id == id)
      .and_then(|session| session.monitors.as_mut())
    else {
      return false;
    };
    selection.icon_path = path;
    selection.active
  });
  if active {
    publish(app, id, "idle");
  }
}

/// Revalidate the selected physical display on release before applying a frame.
pub(super) fn commit(app: &AppHandle, session: &mut Session) {
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
  let original = session.original_frame;
  let frame = monitors::carry_frame(
    GlideFrame {
      x: original.origin.x,
      y: original.origin.y,
      width: original.size.width,
      height: original.size.height,
    },
    selection.displays[selection.source].work,
    destination.work,
  );
  session.work_origin = (destination.work.x, destination.work.y);
  session.work_size = (destination.work.width, destination.work.height);
  if let Some(selection) = session.monitors.as_mut() {
    selection.landing_frame = Some(cidre::cg::Rect {
      origin: cidre::cg::Point {
        x: frame.x,
        y: frame.y,
      },
      size: cidre::cg::Size {
        width: frame.width,
        height: frame.height,
      },
    });
  }
  session.moved = true;
  session.returned_to_origin = false;
  super::super::tween::animate_to(
    &session.target,
    cidre::cg::Rect {
      origin: cidre::cg::Point {
        x: frame.x,
        y: frame.y,
      },
      size: cidre::cg::Size {
        width: frame.width,
        height: frame.height,
      },
    },
    None,
  );
}

pub(super) fn landing(session: &Session) -> Option<CGPoint> {
  let frame = session.monitors.as_ref()?.landing_frame?;
  Some(cursor::landing_point(
    session.anchor,
    session.original_frame,
    frame,
  ))
}
