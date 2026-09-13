// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;

#[test]
fn adjacent_guides_create_a_movable_gap_with_single_step_history() {
  let state = RulerState::default();
  let generation = state.begin();
  let display = DesktopDisplay {
    id: 1,
    origin: Point { x: 0.0, y: 0.0 },
    size: crate::osc::geometry::Size {
      width: 100.0,
      height: 100.0,
    },
    scale: 1.0,
  };
  let image = CapturedImage {
    rgba: vec![0; 100 * 100 * 4],
    width: 100,
    height: 100,
  };
  assert!(state.install(generation, &[display], &[(1, image)]));
  {
    let mut session = state.0.lock().unwrap();
    session.document.guides = vec![
      GuideArtifact {
        id: 1,
        display_id: 1,
        axis: GuideAxis::Vertical,
        position: 20.0,
        anchor: 40.0,
      },
      GuideArtifact {
        id: 2,
        display_id: 1,
        axis: GuideAxis::Vertical,
        position: 60.0,
        anchor: 45.0,
      },
    ];
    session.document.next_id = 2;
    reconcile_guide_gaps(&mut session.document);
  }
  let gap = state.guide_gaps()[0];
  assert_eq!((gap.start, gap.end, gap.position), (20.0, 60.0, 42.5));
  assert_eq!(gap.axis, ProbeAxis::Horizontal);

  let start = state.map_pointer(Point { x: 60.0, y: 70.0 }).unwrap();
  assert!(state.hover(start).is_some());
  assert!(state.pointer_down(start).is_some());
  assert!(state.pointer_up(start).is_some());
  assert!(state.0.lock().unwrap().undo.is_empty());
  assert_eq!(state.guides()[1].position, 60.0);

  assert!(state.hover(start).is_some());
  assert!(state.pointer_down(start).is_some());
  let moved = state.map_pointer(Point { x: 75.0, y: 70.0 }).unwrap();
  assert!(state.pointer_drag(moved).is_some());
  assert!(state.pointer_up(moved).is_some());
  assert_eq!(state.guides()[1].position, 75.0);
  assert_eq!(state.guide_gaps()[0].end, 75.0);
  assert_eq!(state.0.lock().unwrap().undo.len(), 1);
  assert!(state.undo().is_some());
  assert_eq!(state.guides()[1].position, 60.0);
  assert!(state.redo().is_some());
  assert_eq!(state.guides()[1].position, 75.0);

  let gap = state.guide_gaps()[0];
  let label_center = state
    .map_pointer(Point {
      x: (gap.start + gap.end) * 0.5,
      y: gap.position,
    })
    .unwrap();
  assert!(state
    .begin_label_drag(LabelKind::GuideGap, gap.id, label_center, label_center)
    .is_some());
  let label_end = state.map_pointer(Point { x: 58.0, y: 58.0 }).unwrap();
  assert!(state.update_label_drag(label_end).is_some());
  assert!(state.finish_label_drag(label_end).is_some());
  assert_eq!(state.guide_gaps()[0].label_anchor, Some(label_end.world));
  assert_eq!(state.guide_gaps()[0].position, label_end.world.y);
  assert!(state.hide_label(LabelKind::GuideGap, gap.id).is_some());
  assert!(state.guide_gaps()[0].label_hidden);
  assert!(state.undo().is_some());
  assert!(!state.guide_gaps()[0].label_hidden);
}

#[test]
fn deleting_a_guide_gap_removes_its_newer_guide() {
  let state = RulerState::default();
  {
    let mut session = state.0.lock().unwrap();
    session.active = true;
    session.visual = Some(RulerVisual {
      point: Point { x: 40.0, y: 50.0 },
      screen_point: Point { x: 40.0, y: 50.0 },
      display_id: 1,
      zoom: 1.0,
      rgba: [0, 0, 0, 255],
      crosshair: false,
      copied: false,
    });
    session.document.guides = vec![
      GuideArtifact {
        id: 1,
        display_id: 1,
        axis: GuideAxis::Vertical,
        position: 20.0,
        anchor: 50.0,
      },
      GuideArtifact {
        id: 2,
        display_id: 1,
        axis: GuideAxis::Vertical,
        position: 60.0,
        anchor: 50.0,
      },
    ];
    session.document.next_id = 2;
    reconcile_guide_gaps(&mut session.document);
    let gap_id = session.document.guide_gaps[0].id;
    session.hovered_target = Some(HoverTarget::GuideGap(gap_id));
  }
  assert!(state.delete_targeted_artifact().is_some());
  assert_eq!(state.guides().len(), 1);
  assert_eq!(state.guides()[0].id, 1);
  assert!(state.guide_gaps().is_empty());
  assert!(state.undo().is_some());
  assert_eq!(state.guides().len(), 2);
  assert_eq!(state.guide_gaps().len(), 1);
}

#[test]
fn option_clips_transient_probes_to_guides_and_measurement_boxes() {
  let state = RulerState::default();
  let generation = state.begin();
  let display = DesktopDisplay {
    id: 1,
    origin: Point { x: 0.0, y: 0.0 },
    size: crate::osc::geometry::Size {
      width: 100.0,
      height: 100.0,
    },
    scale: 1.0,
  };
  let image = CapturedImage {
    rgba: vec![0; 100 * 100 * 4],
    width: 100,
    height: 100,
  };
  assert!(state.install(generation, &[display], &[(1, image)]));
  {
    let mut session = state.0.lock().unwrap();
    session.document.guides = vec![
      GuideArtifact {
        id: 1,
        display_id: 1,
        axis: GuideAxis::Vertical,
        position: 30.0,
        anchor: 10.0,
      },
      GuideArtifact {
        id: 2,
        display_id: 1,
        axis: GuideAxis::Vertical,
        position: 70.0,
        anchor: 10.0,
      },
      GuideArtifact {
        id: 3,
        display_id: 1,
        axis: GuideAxis::Horizontal,
        position: 20.0,
        anchor: 10.0,
      },
      GuideArtifact {
        id: 4,
        display_id: 1,
        axis: GuideAxis::Horizontal,
        position: 80.0,
        anchor: 10.0,
      },
    ];
    session.document.measurements.push(Measurement {
      id: 5,
      bounds: Rect::from_xywh(35.0, 40.0, 30.0, 20.0),
      label: ArtifactLabel::default(),
    });
    session.document.next_id = 5;
    reconcile_guide_gaps(&mut session.document);
  }
  let pointer = state.map_pointer(Point { x: 50.0, y: 50.0 }).unwrap();
  assert!(state.hover(pointer).is_some());
  let probes = state.probes();
  assert_eq!((probes[0].start, probes[0].end), (0.0, 99.0));
  assert_eq!((probes[1].start, probes[1].end), (0.0, 99.0));
  assert!(state.set_option_active(true).is_some());
  let probes = state.probes();
  assert_eq!((probes[0].start, probes[0].end), (35.0, 65.0));
  assert_eq!((probes[1].start, probes[1].end), (40.0, 60.0));
  assert!(state.set_option_active(false).is_some());
  let probes = state.probes();
  assert_eq!((probes[0].start, probes[0].end), (0.0, 99.0));
  assert_eq!((probes[1].start, probes[1].end), (0.0, 99.0));
}

#[test]
fn tolerance_changes_every_live_edge_detector_and_preserves_stamped_artifacts() {
  let state = RulerState::default();
  let generation = state.begin();
  let display = DesktopDisplay {
    id: 1,
    origin: Point { x: 0.0, y: 0.0 },
    size: crate::osc::geometry::Size {
      width: 100.0,
      height: 100.0,
    },
    scale: 1.0,
  };
  let mut rgba = vec![255; 100 * 100 * 4];
  for y in 30..70 {
    for x in 30..90 {
      let offset = (y * 100 + x) * 4;
      rgba[offset..offset + 4].copy_from_slice(&[248, 249, 250, 255]);
    }
  }
  let image = CapturedImage {
    rgba,
    width: 100,
    height: 100,
  };
  assert!(state.install(generation, &[display], &[(1, image)]));
  let balanced_boxes = state.0.lock().unwrap().boxes.clone();
  assert!(balanced_boxes.is_empty());
  let pointer = state.map_pointer(Point { x: 50.0, y: 50.0 }).unwrap();
  assert!(state.hover(pointer).is_some());
  let horizontal = state
    .probes()
    .into_iter()
    .find(|probe| probe.axis == ProbeAxis::Horizontal)
    .unwrap();
  assert_eq!((horizontal.start, horizontal.end), (0.0, 99.0));

  assert!(state.begin_guide(GuideAxis::Vertical).is_some());
  assert!(state.pointer_down(pointer).is_some());
  assert!(state.cancel_guide().is_some());
  let stamped = state
    .guides()
    .into_iter()
    .map(|guide| (guide.id, guide.display_id, guide.axis, guide.position))
    .collect::<Vec<_>>();
  let probe_pointer = state.map_pointer(Point { x: 60.0, y: 50.0 }).unwrap();
  assert!(state.hover(probe_pointer).is_some());
  {
    let mut session = state.0.lock().unwrap();
    session.document.next_id += 1;
    let id = session.document.next_id;
    session.document.measurements.push(Measurement {
      id,
      bounds: Rect::from_xywh(0.0, 0.0, 10.0, 10.0),
      label: ArtifactLabel::default(),
    });
  }
  let _ = state.center_aids();
  assert!(state.0.lock().unwrap().center_aid_cache.is_some());
  let stamped_document = state.0.lock().unwrap().document.clone();

  assert!(state.cycle_tolerance().is_some());
  assert_eq!(state.tolerance_notice(), Some(Tolerance::SubtleEdges));
  let session = state.0.lock().unwrap();
  let subtle_boxes = session.boxes.clone();
  assert_eq!(subtle_boxes.len(), 1);
  assert_ne!(subtle_boxes, balanced_boxes);
  assert_eq!(session.document, stamped_document);
  assert!(session.center_aid_cache.is_none());
  drop(session);
  assert_eq!(
    state
      .guides()
      .into_iter()
      .map(|guide| (guide.id, guide.display_id, guide.axis, guide.position))
      .collect::<Vec<_>>(),
    stamped
  );
  let horizontal = state
    .probes()
    .into_iter()
    .find(|probe| probe.axis == ProbeAxis::Horizontal)
    .unwrap();
  assert_eq!((horizontal.start, horizontal.end), (30.0, 90.0));

  assert!(state.cycle_tolerance().is_some());
  assert_eq!(state.tolerance_notice(), Some(Tolerance::ClearEdges));
  assert!(state.0.lock().unwrap().boxes.is_empty());
  assert_eq!(
    state
      .guides()
      .into_iter()
      .map(|guide| (guide.id, guide.display_id, guide.axis, guide.position))
      .collect::<Vec<_>>(),
    stamped
  );
  let horizontal = state
    .probes()
    .into_iter()
    .find(|probe| probe.axis == ProbeAxis::Horizontal)
    .unwrap();
  assert_eq!((horizontal.start, horizontal.end), (0.0, 99.0));

  assert!(state.cycle_tolerance().is_some());
  assert_eq!(state.tolerance_notice(), Some(Tolerance::Balanced));
  assert_eq!(state.0.lock().unwrap().boxes, balanced_boxes);
}

#[test]
fn guide_snapping_uses_the_same_tolerance_as_other_live_edges() {
  let state = RulerState::default();
  let generation = state.begin();
  let display = DesktopDisplay {
    id: 1,
    origin: Point { x: 0.0, y: 0.0 },
    size: crate::osc::geometry::Size {
      width: 100.0,
      height: 100.0,
    },
    scale: 1.0,
  };
  let mut rgba = vec![255; 100 * 100 * 4];
  for y in 30..70 {
    for x in 30..90 {
      let offset = (y * 100 + x) * 4;
      rgba[offset..offset + 4].copy_from_slice(&[248, 249, 250, 255]);
    }
  }
  assert!(state.install(
    generation,
    &[display],
    &[(
      1,
      CapturedImage {
        rgba,
        width: 100,
        height: 100,
      },
    )],
  ));
  let pointer = state.map_pointer(Point { x: 31.0, y: 50.0 }).unwrap();
  let session = state.0.lock().unwrap();
  let snapshot = &session.displays[0];
  assert_eq!(
    snap_guide(snapshot, GuideAxis::Vertical, pointer, Tolerance::Balanced),
    None
  );
  assert_eq!(
    snap_guide(
      snapshot,
      GuideAxis::Vertical,
      pointer,
      Tolerance::SubtleEdges,
    ),
    Some(30.0)
  );
}

#[test]
fn crosshair_and_copy_feedback_share_the_current_visual() {
  let visual = RulerVisual {
    point: Point { x: 1.0, y: 2.0 },
    screen_point: Point { x: 1.0, y: 2.0 },
    display_id: 1,
    zoom: 1.0,
    rgba: [0x12, 0xAB, 0xEF, 0xFF],
    crosshair: false,
    copied: false,
  };
  assert_eq!(visual.hex(), "#12ABEF");
  assert_eq!(visual.packed_rgba(), 0x12AB_EFFF);
}
