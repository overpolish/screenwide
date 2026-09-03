// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Display discovery for the Region OSC, the Windows twin of
//! `native_osc_macos/desktop.rs`. All projection and owner selection happens
//! in the shared runtime; this file only turns the monitor list into a
//! [`DesktopBinding`].

use tauri::{Manager, WebviewWindow};

use crate::osc::{
  desktop::{DesktopBinding, DesktopDisplay},
  geometry::{Point, Rect, Size},
};

/// One monitor as Windows reports it: virtual-screen position and size in
/// physical pixels, plus its own DPI scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MonitorProbe {
  pub id: u32,
  pub origin: Point,
  pub size: Size,
  pub scale: f64,
}

/// Enumerates the desktop and binds it to `anchor_id`.
///
/// Ids come from `xcap` so they match the monitor ids the frontend already
/// uses (`recording_sources::list_monitors`); scale factors come from Tauri,
/// which is the only API exposing them. Ordering is the sole cross-API
/// mapping, exactly as in `list_monitors`.
/// One non-anchor display's peer window, in the physical coordinates Win32
/// positions windows with plus the desktop-plane offset its drawing uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PeerPlan {
  pub display_id: u32,
  pub bounds: Rect,
  pub scale: f64,
  pub offset: Point,
}

/// The peer windows a binding calls for: every display except the anchor,
/// which the Tauri window already covers.
pub(crate) fn peer_plan(binding: &DesktopBinding, probes: &[MonitorProbe]) -> Vec<PeerPlan> {
  binding
    .displays
    .iter()
    .filter(|display| display.id != binding.anchor_id)
    .filter_map(|display| {
      let probe = probes.iter().find(|probe| probe.id == display.id)?;
      Some(PeerPlan {
        display_id: display.id,
        bounds: Rect::from_xywh(
          probe.origin.x,
          probe.origin.y,
          probe.size.width,
          probe.size.height,
        ),
        scale: probe.scale,
        offset: display.origin,
      })
    })
    .collect()
}

pub(crate) fn configure_window(
  window: &WebviewWindow,
  anchor_id: u32,
) -> Result<(DesktopBinding, Vec<MonitorProbe>), String> {
  let capture_monitors = xcap::Monitor::all().map_err(|error| {
    let error = error.to_string();
    eprintln!("The Windows region OSC could not enumerate monitors: {error}");
    error
  })?;
  let tauri_monitors = window
    .app_handle()
    .available_monitors()
    .map_err(|error| error.to_string())?;
  if capture_monitors.len() != tauri_monitors.len() {
    let error = "Tauri and xcap returned different monitor counts".to_owned();
    eprintln!("The Windows region OSC could not enumerate monitors: {error}");
    return Err(error);
  }
  let probes = capture_monitors
    .into_iter()
    .zip(tauri_monitors)
    .map(|(monitor, tauri_monitor)| {
      let scale = tauri_monitor.scale_factor();
      let position = tauri_monitor.position();
      let size = tauri_monitor.size();
      Ok(MonitorProbe {
        id: monitor.id().map_err(|error| error.to_string())?,
        origin: Point {
          x: f64::from(position.x),
          y: f64::from(position.y),
        },
        size: Size {
          width: f64::from(size.width),
          height: f64::from(size.height),
        },
        scale,
      })
    })
    .collect::<Result<Vec<_>, String>>()?;
  // The anchor substitution mirrors AppKit's `nearest_screen(host.frame)`:
  // the display closest to the window that lost its monitor.
  let hint = host_rect(window);
  let binding = build_binding(&probes, anchor_id, hint)?;
  if let Some(anchor) = probes
    .iter()
    .find(|monitor| monitor.id == binding.anchor_id)
  {
    place_over_anchor(window, *anchor);
  }
  Ok((binding, probes))
}

/// The desktop compositor owns the host window's placement, exactly as the
/// macOS `rebuild_surfaces` sets the parent frame to the anchor screen:
/// `show_region_selector` deliberately skips sizing in desktop mode.
fn place_over_anchor(window: &WebviewWindow, anchor: MonitorProbe) {
  let target = Rect::from_xywh(
    anchor.origin.x,
    anchor.origin.y,
    anchor.size.width,
    anchor.size.height,
  );
  if host_rect(window) == Some(target) {
    return;
  }
  let position = tauri::PhysicalPosition::new(anchor.origin.x as i32, anchor.origin.y as i32);
  let size = tauri::PhysicalSize::new(anchor.size.width as u32, anchor.size.height as u32);
  if window.set_position(position).is_err() || window.set_size(size).is_err() {
    eprintln!("The Windows region OSC could not cover the anchor monitor");
  }
}

fn host_rect(window: &WebviewWindow) -> Option<Rect> {
  let position = window.outer_position().ok()?;
  let size = window.outer_size().ok()?;
  Some(Rect::from_xywh(
    f64::from(position.x),
    f64::from(position.y),
    f64::from(size.width),
    f64::from(size.height),
  ))
}

/// Builds the binding from physical monitor geometry.
///
/// The desktop plane is logical points: every display is divided by its own
/// scale and the union is normalized so its top-left is the origin. The
/// portable math is pure translation with no scale terms, so this keeps a
/// single monitor exact and makes mixed-DPI adjacency approximate until the
/// peer surfaces land.
///
/// `layout_changed` is left false here; the caller owns the comparison against
/// the previously configured binding.
pub(crate) fn build_binding(
  monitors: &[MonitorProbe],
  anchor_id: u32,
  hint: Option<Rect>,
) -> Result<DesktopBinding, String> {
  let valid = monitors
    .iter()
    .copied()
    .filter(|monitor| {
      monitor.origin.finite()
        && monitor.size.valid()
        && monitor.size.width > 0.0
        && monitor.size.height > 0.0
        && monitor.scale.is_finite()
        && monitor.scale > 0.0
    })
    .collect::<Vec<_>>();
  if valid.is_empty() {
    return Err("Windows returned no valid desktop displays".to_owned());
  }
  let resolved_anchor_id = if valid.iter().any(|monitor| monitor.id == anchor_id) {
    anchor_id
  } else {
    nearest(&valid, hint).map_or(anchor_id, |monitor| monitor.id)
  };

  // Physical origins can be negative; the plane is normalized so a region's
  // desktop coordinates are never negative either.
  let logical = valid
    .iter()
    .map(|monitor| DesktopDisplay {
      id: monitor.id,
      origin: Point {
        x: monitor.origin.x / monitor.scale,
        y: monitor.origin.y / monitor.scale,
      },
      size: Size {
        width: monitor.size.width / monitor.scale,
        height: monitor.size.height / monitor.scale,
      },
      scale: monitor.scale,
    })
    .collect::<Vec<_>>();
  let min_x = logical
    .iter()
    .map(|display| display.origin.x)
    .fold(f64::INFINITY, f64::min);
  let min_y = logical
    .iter()
    .map(|display| display.origin.y)
    .fold(f64::INFINITY, f64::min);
  let displays = logical
    .into_iter()
    .map(|display| DesktopDisplay {
      origin: Point {
        x: display.origin.x - min_x,
        y: display.origin.y - min_y,
      },
      ..display
    })
    .collect::<Vec<_>>();
  if !displays
    .iter()
    .any(|display| display.id == resolved_anchor_id)
  {
    return Err(format!(
      "Windows could not resolve a Region monitor after losing: {anchor_id}"
    ));
  }
  let size = Size {
    width: displays
      .iter()
      .map(|display| display.origin.x + display.size.width)
      .fold(0.0, f64::max),
    height: displays
      .iter()
      .map(|display| display.origin.y + display.size.height)
      .fold(0.0, f64::max),
  };
  if !size.valid() || size.width <= 0.0 || size.height <= 0.0 {
    return Err("Windows returned no valid desktop displays".to_owned());
  }
  Ok(DesktopBinding {
    displays,
    anchor_id: resolved_anchor_id,
    size,
    layout_changed: false,
  })
}

/// Squared-distance pick in physical virtual-screen space, the same metric the
/// shared `reconcile_region` uses for stranded regions.
fn nearest(monitors: &[MonitorProbe], hint: Option<Rect>) -> Option<MonitorProbe> {
  let Some(hint) = hint else {
    return monitors.first().copied();
  };
  monitors.iter().copied().min_by(|a, b| {
    let distance = |monitor: MonitorProbe| {
      let right = monitor.origin.x + monitor.size.width;
      let bottom = monitor.origin.y + monitor.size.height;
      let dx = if hint.right() < monitor.origin.x {
        monitor.origin.x - hint.right()
      } else if hint.origin.x > right {
        hint.origin.x - right
      } else {
        0.0
      };
      let dy = if hint.bottom() < monitor.origin.y {
        monitor.origin.y - hint.bottom()
      } else if hint.origin.y > bottom {
        hint.origin.y - bottom
      } else {
        0.0
      };
      dx * dx + dy * dy
    };
    distance(*a).total_cmp(&distance(*b))
  })
}

pub(crate) fn global_committed(binding: &DesktopBinding, local: Option<Rect>) -> Option<Rect> {
  local.and_then(|region| binding.project_local(region))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn probe(id: u32, x: f64, y: f64, width: f64, height: f64, scale: f64) -> MonitorProbe {
    MonitorProbe {
      id,
      origin: Point { x, y },
      size: Size { width, height },
      scale,
    }
  }

  #[test]
  fn one_monitor_becomes_its_own_logical_desktop() {
    let binding = build_binding(&[probe(7, 0.0, 0.0, 1920.0, 1080.0, 1.0)], 7, None).unwrap();

    assert_eq!(binding.anchor_id, 7);
    assert_eq!(binding.displays.len(), 1);
    assert_eq!(binding.displays[0].origin, Point { x: 0.0, y: 0.0 });
    assert_eq!(
      binding.size,
      Size {
        width: 1920.0,
        height: 1080.0
      }
    );
  }

  #[test]
  fn a_scaled_monitor_reports_its_logical_extent() {
    let binding = build_binding(&[probe(1, 0.0, 0.0, 2880.0, 1620.0, 1.5)], 1, None).unwrap();

    assert_eq!(
      binding.size,
      Size {
        width: 1920.0,
        height: 1080.0
      }
    );
    assert_eq!(binding.displays[0].scale, 1.5);
    assert_eq!(binding.virtual_monitor().size, binding.size);
  }

  #[test]
  fn a_monitor_left_of_the_primary_normalizes_the_union_to_the_origin() {
    let binding = build_binding(
      &[
        probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0),
        probe(2, -1280.0, -100.0, 1280.0, 1024.0, 1.0),
      ],
      1,
      None,
    )
    .unwrap();

    // The left monitor lands at the origin and the primary shifts right.
    assert_eq!(binding.displays[1].origin, Point { x: 0.0, y: 0.0 });
    assert_eq!(
      binding.displays[0].origin,
      Point {
        x: 1280.0,
        y: 100.0
      }
    );
    assert_eq!(
      binding.size,
      Size {
        width: 3200.0,
        height: 1180.0
      }
    );
    // A region local to the anchor projects past the seam without clamping.
    assert_eq!(
      binding.project_local(Rect::from_xywh(-200.0, 0.0, 400.0, 300.0)),
      Some(Rect::from_xywh(1080.0, 100.0, 400.0, 300.0))
    );
  }

  #[test]
  fn a_lost_anchor_is_replaced_by_the_display_nearest_the_window() {
    let monitors = [
      probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0),
      probe(2, 1920.0, 0.0, 1920.0, 1080.0, 1.0),
    ];

    let near_right = build_binding(
      &monitors,
      99,
      Some(Rect::from_xywh(2000.0, 100.0, 800.0, 600.0)),
    )
    .unwrap();
    assert_eq!(near_right.anchor_id, 2);

    let near_left = build_binding(
      &monitors,
      99,
      Some(Rect::from_xywh(-400.0, 100.0, 300.0, 200.0)),
    )
    .unwrap();
    assert_eq!(near_left.anchor_id, 1);

    // Without a hint the first display stands in.
    assert_eq!(build_binding(&monitors, 99, None).unwrap().anchor_id, 1);
  }

  #[test]
  fn an_empty_or_degenerate_desktop_is_refused() {
    assert!(build_binding(&[], 1, None).is_err());
    assert!(build_binding(&[probe(1, 0.0, 0.0, 0.0, 1080.0, 1.0)], 1, None).is_err());
    assert!(build_binding(&[probe(1, 0.0, 0.0, 1920.0, 1080.0, 0.0)], 1, None).is_err());
  }

  #[test]
  fn the_anchor_never_gets_a_peer_and_every_other_display_does() {
    let monitors = [
      probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0),
      probe(2, 1920.0, 0.0, 2560.0, 1440.0, 2.0),
      probe(3, -1280.0, 0.0, 1280.0, 1024.0, 1.0),
    ];
    let binding = build_binding(&monitors, 1, None).unwrap();

    let plan = peer_plan(&binding, &monitors);
    assert_eq!(
      plan.iter().map(|peer| peer.display_id).collect::<Vec<_>>(),
      vec![2, 3]
    );
    // Peer windows are positioned in physical pixels...
    let scaled = plan.iter().find(|peer| peer.display_id == 2).unwrap();
    assert_eq!(scaled.bounds, Rect::from_xywh(1920.0, 0.0, 2560.0, 1440.0));
    assert_eq!(scaled.scale, 2.0);
    // ...but their drawing offset is the normalized desktop plane. Dividing
    // each monitor by its own scale is what makes mixed-DPI adjacency
    // approximate: the 200% display starts at 1920/2 = 960 physical-derived
    // points, so in the plane it overlaps its neighbour instead of abutting it.
    assert_eq!(scaled.offset, Point { x: 2240.0, y: 0.0 });
    let left = plan.iter().find(|peer| peer.display_id == 3).unwrap();
    assert_eq!(left.offset, Point { x: 0.0, y: 0.0 });

    // A single-monitor desktop plans no peers at all, which is what keeps
    // desktop presentation a no-op there.
    let single = build_binding(&monitors[..1], 1, None).unwrap();
    assert!(peer_plan(&single, &monitors).is_empty());
  }

  #[test]
  fn a_display_without_physical_geometry_is_skipped_rather_than_guessed() {
    let monitors = [
      probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0),
      probe(2, 1920.0, 0.0, 1920.0, 1080.0, 1.0),
    ];
    let binding = build_binding(&monitors, 1, None).unwrap();

    assert!(peer_plan(&binding, &monitors[..1]).is_empty());
  }

  #[test]
  fn layout_changes_are_detected_from_the_previous_binding() {
    let first = build_binding(&[probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0)], 1, None).unwrap();
    let same = build_binding(&[probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0)], 1, None).unwrap();
    let resized = build_binding(&[probe(1, 0.0, 0.0, 2560.0, 1440.0, 1.0)], 1, None).unwrap();

    assert!(super::super::state::layout_changed(None, &first));
    assert!(!super::super::state::layout_changed(
      Some(&(first.displays.clone(), first.anchor_id)),
      &same
    ));
    assert!(super::super::state::layout_changed(
      Some(&(first.displays.clone(), first.anchor_id)),
      &resized
    ));
    assert!(super::super::state::layout_changed(
      Some(&(first.displays.clone(), 9)),
      &first
    ));
  }
}
