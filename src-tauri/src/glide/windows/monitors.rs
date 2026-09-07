// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows monitor topology and selection. Geometry is paired through the
//! shared capture/native topology snapshot so monitor IDs survive rearranging.

use crate::glide::core::{monitors::Monitor, GlideFrame};
use tauri::AppHandle;
use windows::Win32::Foundation::POINT;

pub(super) struct Selection {
  pub displays: Vec<Monitor>,
  pub source: usize,
  pub selected: usize,
}

pub(super) fn capture(app: &AppHandle, anchor: POINT) -> Option<Selection> {
  let displays = snapshot(app)?;
  if displays.is_empty() {
    return None;
  }
  let source = displays.iter().position(|monitor| {
    let r = monitor.bounds;
    f64::from(anchor.x) >= r.x
      && f64::from(anchor.x) < r.x + r.width
      && f64::from(anchor.y) >= r.y
      && f64::from(anchor.y) < r.y + r.height
  })?;
  Some(Selection {
    displays,
    source,
    selected: source,
  })
}

fn snapshot(app: &AppHandle) -> Option<Vec<Monitor>> {
  let topology = crate::monitor_topology::snapshot(app).ok()?.into_iter();
  let mut displays = Vec::new();
  for display in topology {
    let native = display.native;
    let position = native.position();
    let size = native.size();
    let work = native.work_area();
    let work_position = work.position;
    let work_size = work.size;
    let monitor = Monitor {
      id: display.id.to_string(),
      layout: GlideFrame {
        x: display.layout_position.0,
        y: display.layout_position.1,
        width: display.layout_size.0,
        height: display.layout_size.1,
      },
      bounds: GlideFrame {
        x: f64::from(position.x),
        y: f64::from(position.y),
        width: f64::from(size.width),
        height: f64::from(size.height),
      },
      work: GlideFrame {
        x: f64::from(work_position.x),
        y: f64::from(work_position.y),
        width: f64::from(work_size.width),
        height: f64::from(work_size.height),
      },
    };
    if !displays
      .iter()
      .any(|item: &Monitor| item.bounds == monitor.bounds)
    {
      displays.push(monitor);
    }
  }
  Some(displays)
}

pub(super) fn commit_frame(
  app: &AppHandle,
  selection: &Selection,
  original: GlideFrame,
) -> Option<GlideFrame> {
  if selection.selected == selection.source {
    return None;
  }
  let fresh = snapshot(app)?;
  let id = &selection.displays[selection.selected].id;
  let destination = fresh.iter().find(|monitor| &monitor.id == id)?.work;
  Some(crate::glide::core::monitors::carry_frame(
    original,
    selection.displays[selection.source].work,
    destination,
  ))
}
