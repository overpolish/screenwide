// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;

#[test]
fn dismissal_invalidates_topology_restarts() {
  let state = RulerState::default();
  let generation = state.begin();
  assert_eq!(state.active_generation(), Some(generation));
  assert!(state.cancel());
  assert_eq!(state.active_generation(), None);
  assert!(!state.is_current(generation));
}

#[test]
fn radius_candidates_are_cached_for_each_tolerance() {
  let width = 120u32;
  let height = 120u32;
  let mut rgba = vec![0xFF; (width * height * 4) as usize];
  for pixel in rgba.chunks_mut(4) {
    pixel[3] = 255;
  }
  let set = |rgba: &mut [u8], x: u32, y: u32, rgb: [u8; 3]| {
    let index = ((y * width + x) * 4) as usize;
    rgba[index..index + 3].copy_from_slice(&rgb);
  };
  for y in 30..70 {
    for x in 30..90 {
      set(&mut rgba, x, y, [0xF8, 0xF9, 0xFA]);
    }
  }
  let blend = [0xFC, 0xFC, 0xFD];
  for x in 30..90 {
    set(&mut rgba, x, 30, blend);
    set(&mut rgba, x, 69, blend);
  }
  for y in 30..70 {
    set(&mut rgba, 30, y, blend);
    set(&mut rgba, 89, y, blend);
  }
  let state = RulerState::default();
  let generation = state.begin();
  let display = DesktopDisplay {
    id: 1,
    origin: Point { x: 0.0, y: 0.0 },
    size: crate::osc::geometry::Size {
      width: f64::from(width),
      height: f64::from(height),
    },
    scale: 1.0,
  };
  assert!(state.install(
    generation,
    &[display],
    &[(
      1,
      CapturedImage {
        rgba,
        width,
        height,
      },
    )],
  ));
  let session = state.0.lock().unwrap();
  let snapshot = &session.displays[0];
  assert!(snapshot.boxes_by_tolerance[Tolerance::ClearEdges.index()].is_empty());
  assert!(snapshot.boxes_by_tolerance[Tolerance::Balanced.index()].is_empty());
  assert_eq!(
    snapshot.boxes_by_tolerance[Tolerance::SubtleEdges.index()].len(),
    1
  );
}

#[test]
fn samples_each_display_in_its_own_pixel_density() {
  let state = RulerState::default();
  let generation = state.begin();
  let displays = [
    DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 2.0,
        height: 1.0,
      },
      scale: 2.0,
    },
    DesktopDisplay {
      id: 2,
      origin: Point { x: 2.0, y: 1.0 },
      size: crate::osc::geometry::Size {
        width: 1.0,
        height: 1.0,
      },
      scale: 1.0,
    },
  ];
  let snapshots = [
    (
      1,
      CapturedImage {
        rgba: vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255],
        width: 4,
        height: 1,
      },
    ),
    (
      2,
      CapturedImage {
        rgba: vec![20, 21, 22, 255],
        width: 1,
        height: 1,
      },
    ),
  ];
  assert!(state.install(generation, &displays, &snapshots));
  assert_eq!(
    state
      .hover(state.map_pointer(Point { x: 1.75, y: 0.5 }).unwrap())
      .unwrap()
      .rgba,
    [10, 11, 12, 255]
  );
  assert_eq!(
    state
      .hover(state.map_pointer(Point { x: 2.5, y: 1.5 }).unwrap())
      .unwrap()
      .rgba,
    [20, 21, 22, 255]
  );
  assert!(state.map_pointer(Point { x: 2.5, y: 0.5 }).is_none());
}

#[test]
fn viewport_actions_change_only_the_target_display() {
  let state = RulerState::default();
  let generation = state.begin();
  let displays = [
    DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 100.0,
        height: 80.0,
      },
      scale: 1.0,
    },
    DesktopDisplay {
      id: 2,
      origin: Point { x: 100.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 100.0,
        height: 80.0,
      },
      scale: 2.0,
    },
  ];
  let image = CapturedImage {
    rgba: vec![0; 100 * 80 * 4],
    width: 100,
    height: 80,
  };
  assert!(state.install(generation, &displays, &[(1, image.clone()), (2, image)]));
  assert!(state
    .hover(state.map_pointer(Point { x: 150.0, y: 40.0 }).unwrap())
    .is_some());
  state.update_viewport(
    2,
    ViewportAction::Zoom {
      anchor: Point { x: 50.0, y: 40.0 },
      factor: 2.0,
    },
  );
  let viewports = state.viewports();
  assert_eq!(viewports[0].viewport, Viewport::default());
  assert_eq!(viewports[1].viewport.zoom, 2.0);
  assert_eq!(
    state.map_pointer(Point { x: 100.0, y: 0.0 }).unwrap().world,
    Point { x: 125.0, y: 20.0 }
  );
  state.update_viewport(
    2,
    ViewportAction::Reset {
      anchor: Point { x: 0.0, y: 0.0 },
    },
  );
  assert!(state
    .viewports()
    .iter()
    .all(|item| item.viewport == Viewport::default()));
}

#[test]
fn live_probes_use_frozen_source_edges_and_hide_during_pointer_gestures() {
  let state = RulerState::default();
  let generation = state.begin();
  let display = DesktopDisplay {
    id: 7,
    origin: Point { x: 100.0, y: 50.0 },
    size: crate::osc::geometry::Size {
      width: 10.0,
      height: 5.0,
    },
    scale: 2.0,
  };
  let mut rgba = vec![0; 20 * 10 * 4];
  for y in 2..9 {
    for x in 3..16 {
      let offset = (y * 20 + x) * 4;
      rgba[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
    }
  }
  let image = CapturedImage {
    rgba,
    width: 20,
    height: 10,
  };
  assert!(state.install(generation, &[display], &[(7, image)]));
  let pointer = state.map_pointer(Point { x: 105.2, y: 52.75 }).unwrap();
  assert!(state.hover(pointer).is_some());
  let probes = state.probes();
  assert_eq!(probes.len(), 2);
  assert_eq!(probes[0].display_id, 7);
  assert_eq!(probes[0].axis, ProbeAxis::Horizontal);
  assert_eq!((probes[0].start, probes[0].end), (101.5, 108.0));
  assert_eq!(probes[0].position, 52.75);
  assert_eq!(probes[1].axis, ProbeAxis::Vertical);
  assert_eq!((probes[1].start, probes[1].end), (51.0, 54.5));
  assert_eq!(probes[1].position, 105.2);

  assert!(state.pointer_down(pointer).is_some());
  assert!(state.probes().is_empty());
  assert!(state.cancel_pointer().is_some());
  assert_eq!(state.probes().len(), 2);

  let end = state.map_pointer(Point { x: 109.5, y: 54.5 }).unwrap();
  assert!(state.pointer_down(pointer).is_some());
  assert!(state.pointer_drag(end).is_some());
  assert!(state.pointer_up(end).is_some());
  assert_eq!(state.measurements().len(), 1);
}

#[test]
fn held_range_stamps_a_probe_and_joins_history_copy_and_deletion() {
  let state = RulerState::default();
  let generation = state.begin();
  let display = DesktopDisplay {
    id: 1,
    origin: Point { x: 0.0, y: 0.0 },
    size: crate::osc::geometry::Size {
      width: 10.0,
      height: 5.0,
    },
    scale: 2.0,
  };
  let mut rgba = vec![0; 20 * 10 * 4];
  for y in 2..9 {
    for x in 3..16 {
      let offset = (y * 20 + x) * 4;
      rgba[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
    }
  }
  let image = CapturedImage {
    rgba,
    width: 20,
    height: 10,
  };
  assert!(state.install(generation, &[display], &[(1, image)]));
  let start = state.map_pointer(Point { x: 4.0, y: 2.25 }).unwrap();
  assert!(state.hover(start).is_some());
  assert!(state.begin_range(RangeAxis::Horizontal).is_some());
  let draft = state
    .probes()
    .into_iter()
    .find(|probe| probe.draft)
    .unwrap();
  assert_eq!(draft.axis, ProbeAxis::Horizontal);
  assert_eq!(draft.position, 2.25);

  let end = state.map_pointer(Point { x: 7.0, y: 3.25 }).unwrap();
  assert!(state.pointer_down(start).is_some());
  assert!(state.pointer_drag(end).is_some());
  let live = state
    .probes()
    .into_iter()
    .find(|probe| probe.draft)
    .unwrap();
  assert_eq!((live.start, live.end, live.position), (1.5, 8.0, 3.25));
  assert!(state.pointer_up(end).is_some());
  assert!(state.finish_range().is_some());
  let stamped = state
    .probes()
    .into_iter()
    .find(|probe| probe.id != 0)
    .unwrap();
  assert!(!stamped.draft);
  assert_eq!(
    (stamped.start, stamped.end, stamped.position),
    (1.5, 8.0, 3.25)
  );
  assert_eq!(state.copy_latest_artifact().unwrap().1, "7 px");

  assert!(state.undo().is_some());
  assert!(state.probes().iter().all(|probe| probe.id == 0));
  assert!(state.redo().is_some());
  assert!(state.probes().iter().any(|probe| probe.id != 0));
  assert!(state.delete_targeted_artifact().is_some());
  assert!(state.probes().iter().all(|probe| probe.id == 0));
}

#[test]
fn held_guides_snap_with_hysteresis_stamp_and_join_history() {
  let state = RulerState::default();
  let generation = state.begin();
  let display = DesktopDisplay {
    id: 7,
    origin: Point { x: 100.0, y: 50.0 },
    size: crate::osc::geometry::Size {
      width: 100.0,
      height: 80.0,
    },
    scale: 1.0,
  };
  let mut rgba = vec![0; 100 * 80 * 4];
  for y in 0..80 {
    for x in 0..100 {
      let value = if x >= 40 || y >= 25 { 255 } else { 0 };
      let offset = (y * 100 + x) * 4;
      rgba[offset..offset + 4].copy_from_slice(&[value, value, value, 255]);
    }
  }
  let image = CapturedImage {
    rgba,
    width: 100,
    height: 80,
  };
  assert!(state.install(generation, &[display], &[(7, image)]));

  let start = state.map_pointer(Point { x: 135.0, y: 70.0 }).unwrap();
  assert!(state.hover(start).is_some());
  assert!(state.begin_guide(GuideAxis::Vertical).is_some());
  assert_eq!(state.guides()[0].position, 140.0);

  let retained = state.map_pointer(Point { x: 150.0, y: 70.0 }).unwrap();
  assert!(state.hover(retained).is_some());
  assert_eq!(state.guides()[0].position, 140.0);

  let released = state.map_pointer(Point { x: 158.0, y: 70.0 }).unwrap();
  assert!(state.hover(released).is_some());
  assert_eq!(state.guides()[0].position, 158.0);

  assert!(state.hover(start).is_some());
  assert_eq!(state.guides()[0].position, 140.0);
  assert!(state.pointer_down(start).is_some());
  let guides = state.guides();
  assert_eq!(guides.len(), 2);
  assert!(guides.iter().any(|guide| guide.id != 0 && !guide.draft));
  assert!(state.cancel_guide().is_some());
  assert_eq!(state.guides().len(), 1);

  assert!(state.undo().is_some());
  assert!(state.guides().is_empty());
  assert!(state.redo().is_some());
  assert_eq!(state.guides().len(), 1);

  assert!(state.begin_guide(GuideAxis::Horizontal).is_some());
  assert_eq!(state.guides().last().unwrap().position, 75.0);
  assert!(state.cancel_guide().is_some());
}
