// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;

#[test]
fn snapping_prefers_the_components_circled_by_the_drag() {
  let boxes = [
    Rect::from_xywh(20.0, 20.0, 30.0, 20.0),
    Rect::from_xywh(60.0, 20.0, 20.0, 20.0),
  ];
  assert_eq!(
    snap_bounds(&boxes, Rect::from_xywh(15.0, 15.0, 70.0, 30.0)),
    Rect::from_xywh(20.0, 20.0, 60.0, 20.0)
  );
}

#[test]
fn snapping_never_grows_to_an_overlapping_container() {
  let boxes = [Rect::from_xywh(20.0, 20.0, 80.0, 60.0)];
  let drag = Rect::from_xywh(24.0, 24.0, 70.0, 50.0);
  assert_eq!(snap_bounds(&boxes, drag), drag);
}

#[test]
fn snapping_ignores_nearby_edges_outside_the_drawn_bounds() {
  let boxes = [Rect::from_xywh(7.0, 20.0, 106.0, 60.0)];
  let drag = Rect::from_xywh(10.0, 25.0, 100.0, 50.0);

  assert_eq!(snap_bounds(&boxes, drag), drag);
}

#[test]
fn settle_animation_starts_at_the_raw_drag_and_lands_on_the_snap() {
  let from = Rect::from_xywh(10.0, 10.0, 30.0, 30.0);
  let to = Rect::from_xywh(8.0, 8.0, 34.0, 34.0);
  let mut session = Session {
    active: true,
    document: Document {
      measurements: vec![Measurement {
        id: 1,
        bounds: to,
        label: ArtifactLabel::default(),
      }],
      probes: Vec::new(),
      guides: Vec::new(),
      guide_gaps: Vec::new(),
      radii: Vec::new(),
      next_id: 1,
    },
    settle: Some(Settle {
      id: 1,
      from,
      to,
      started: Instant::now(),
    }),
    ..Default::default()
  };
  let first = measurement_visuals(&mut session, Instant::now())[0];
  assert!(first.animating);
  assert!((first.bounds.origin.x - from.origin.x).abs() < 0.1);
  let landed = measurement_visuals(
    &mut session,
    Instant::now() + SETTLE_DURATION + Duration::from_millis(1),
  )[0];
  assert_eq!(landed.bounds, to);
  assert!(!landed.animating);
}

#[test]
fn artifact_hover_targets_the_latest_overlapping_border() {
  let measurements = [
    Measurement {
      id: 1,
      bounds: Rect::from_xywh(10.0, 10.0, 50.0, 50.0),
      label: ArtifactLabel::default(),
    },
    Measurement {
      id: 2,
      bounds: Rect::from_xywh(10.0, 10.0, 50.0, 50.0),
      label: ArtifactLabel::default(),
    },
  ];
  assert_eq!(
    hit_test_measurement(&measurements, Point { x: 10.0, y: 30.0 }, 6.0),
    Some(2)
  );
  assert_eq!(
    hit_test_measurement(&measurements, Point { x: 30.0, y: 30.0 }, 6.0),
    None
  );
}

#[test]
fn hover_target_clears_immediately_while_its_halo_fades_out() {
  let now = Instant::now();
  let target = HoverTarget::Measurement(1);
  let mut session = Session {
    hovered_target: Some(target),
    document: Document {
      measurements: vec![Measurement {
        id: 1,
        bounds: Rect::from_xywh(10.0, 10.0, 20.0, 20.0),
        label: ArtifactLabel::default(),
      }],
      next_id: 1,
      ..Default::default()
    },
    ..Default::default()
  };

  update_hover_target(&mut session, None, now);
  assert_eq!(session.hovered_target, None);
  assert!(session.hover_exit.is_some());
  assert_eq!(hover_alpha(&session, target, now), 1.0);
  let halfway = hover_alpha(&session, target, now + HOVER_EXIT_DURATION / 2);
  assert!(halfway > 0.0 && halfway < 1.0);
  assert_eq!(measurement_visuals(&mut session, now)[0].hover_alpha, 1.0);

  expire_hover_exit(&mut session, now + HOVER_EXIT_DURATION);
  assert!(session.hover_exit.is_none());
  assert_eq!(
    hover_alpha(&session, target, now + HOVER_EXIT_DURATION),
    0.0
  );
}

#[test]
fn label_hover_targets_delete_without_changing_latest_copy() {
  let state = RulerState::default();
  {
    let mut session = state.0.lock().unwrap();
    session.active = true;
    session.visual = Some(RulerVisual {
      point: Point { x: 0.0, y: 0.0 },
      screen_point: Point { x: 0.0, y: 0.0 },
      display_id: 1,
      zoom: 1.0,
      rgba: [0, 0, 0, 255],
      crosshair: false,
      copied: false,
    });
    session.document.measurements = vec![
      Measurement {
        id: 1,
        bounds: Rect::from_xywh(0.0, 0.0, 20.0, 30.0),
        label: ArtifactLabel::default(),
      },
      Measurement {
        id: 2,
        bounds: Rect::from_xywh(40.0, 40.0, 50.0, 60.0),
        label: ArtifactLabel::default(),
      },
    ];
    session.document.next_id = 2;
  }
  assert!(state.hover_measurement_label(1).is_some());
  let visuals = state.measurements();
  assert!(visuals.iter().find(|item| item.id == 1).unwrap().hovered);
  assert_eq!(state.copy_latest_artifact().unwrap().1, "50 × 60 px");
  assert!(state.delete_targeted_artifact().is_some());
  assert_eq!(
    state
      .measurements()
      .into_iter()
      .map(|item| item.id)
      .collect::<Vec<_>>(),
    vec![2]
  );
}

#[test]
fn label_drag_and_visibility_share_document_history() {
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
    rgba: vec![0, 0, 0, 255],
    width: 1,
    height: 1,
  };
  assert!(state.install(generation, &[display], &[(1, image)]));
  let initial = state.map_pointer(Point { x: 30.0, y: 30.0 }).unwrap();
  assert!(state.hover(initial).is_some());
  {
    let mut session = state.0.lock().unwrap();
    session.document.measurements.push(Measurement {
      id: 1,
      bounds: Rect::from_xywh(20.0, 20.0, 40.0, 40.0),
      label: ArtifactLabel::default(),
    });
    session.document.next_id = 1;
  }
  let center = state.map_pointer(Point { x: 40.0, y: 40.0 }).unwrap();
  assert!(state
    .begin_label_drag(LabelKind::Measurement, 1, initial, center)
    .is_some());
  let below_threshold = state.map_pointer(Point { x: 32.0, y: 30.0 }).unwrap();
  assert!(state.update_label_drag(below_threshold).is_some());
  assert!(state.measurements()[0].label_anchor.is_none());
  let moved = state.map_pointer(Point { x: 40.0, y: 30.0 }).unwrap();
  assert!(state.update_label_drag(moved).is_some());
  assert!(state.finish_label_drag(moved).is_some());
  assert_eq!(
    state.measurements()[0].label_anchor,
    Some(Point { x: 50.0, y: 40.0 })
  );
  assert!(state.hide_label(LabelKind::Measurement, 1).is_some());
  assert!(state.measurements()[0].label_hidden);
  let border = state.map_pointer(Point { x: 20.0, y: 40.0 }).unwrap();
  assert!(state.toggle_label_at(border).is_some());
  assert!(!state.measurements()[0].label_hidden);
  assert!(state.toggle_label_at(border).is_some());
  assert!(state.measurements()[0].label_hidden);
  assert!(state.undo().is_some());
  assert!(!state.measurements()[0].label_hidden);
  assert!(state.undo().is_some());
  assert!(state.measurements()[0].label_hidden);
  assert!(state.undo().is_some());
  assert!(!state.measurements()[0].label_hidden);
  assert!(state.undo().is_some());
  assert!(state.measurements()[0].label_anchor.is_none());
}

#[test]
fn history_is_bounded_and_new_edits_clear_redo() {
  let mut session = Session::default();
  for id in 1..=105 {
    record_history(&mut session);
    session.document.measurements.push(Measurement {
      id,
      bounds: Rect::from_xywh(id as f64, 0.0, 10.0, 10.0),
      label: ArtifactLabel::default(),
    });
  }
  assert_eq!(session.undo.len(), HISTORY_LIMIT);
  session.redo.push(Document::default());
  record_history(&mut session);
  assert!(session.redo.is_empty());
}

#[test]
fn undo_redo_and_delete_operate_on_the_artifact_document() {
  let state = RulerState::default();
  {
    let mut session = state.0.lock().unwrap();
    session.active = true;
    session.visual = Some(RulerVisual {
      point: Point { x: 10.0, y: 10.0 },
      screen_point: Point { x: 10.0, y: 10.0 },
      display_id: 1,
      zoom: 1.0,
      rgba: [0, 0, 0, 255],
      crosshair: false,
      copied: false,
    });
    record_history(&mut session);
    session.document.measurements.push(Measurement {
      id: 1,
      bounds: Rect::from_xywh(1.0, 2.0, 30.0, 40.0),
      label: ArtifactLabel::default(),
    });
    session.document.next_id = 1;
    session.hovered_target = Some(HoverTarget::Measurement(1));
  }
  assert_eq!(state.copy_latest_artifact().unwrap().1, "30 × 40 px");
  assert!(state.delete_targeted_artifact().is_some());
  assert!(state.measurements().is_empty());
  assert!(state.undo().is_some());
  assert_eq!(state.measurements().len(), 1);
  assert!(state.redo().is_some());
  assert!(state.measurements().is_empty());
}

#[test]
fn radius_artifacts_share_hit_text_label_and_history_semantics() {
  let radius = RadiusArtifact {
    id: 7,
    display_id: 1,
    bounds: Rect::from_xywh(10.0, 10.0, 80.0, 60.0),
    corner: Corner::TopLeft,
    radius: 12.0,
    low_confidence: true,
    label: ArtifactLabel::default(),
  };
  let mut document = Document {
    radii: vec![radius],
    next_id: 7,
    ..Default::default()
  };
  assert_eq!(latest_target(&document), Some(HoverTarget::Radius(7)));
  assert_eq!(
    artifact_text(&document, HoverTarget::Radius(7)).as_deref(),
    Some("≈ 12 px")
  );
  assert_eq!(
    hit_test_radius(&document.radii, Point { x: 14.0, y: 14.0 }, 3.0),
    Some(7)
  );
  assert_eq!(
    hit_test_radius(&document.radii, Point { x: 60.0, y: 40.0 }, 3.0),
    None
  );
  label_state_mut(&mut document, HoverTarget::Radius(7))
    .unwrap()
    .anchor = Some(Point { x: 30.0, y: 25.0 });
  assert_eq!(
    label_state(&document, HoverTarget::Radius(7))
      .unwrap()
      .anchor,
    Some(Point { x: 30.0, y: 25.0 })
  );
}

#[test]
fn center_aids_are_document_cached_and_toggle_as_one_native_view() {
  let state = RulerState::default();
  {
    let mut session = state.0.lock().unwrap();
    session.active = true;
    session.centerlines_visible = true;
    session.visual = Some(RulerVisual {
      point: Point { x: 40.0, y: 40.0 },
      screen_point: Point { x: 40.0, y: 40.0 },
      display_id: 1,
      zoom: 1.0,
      rgba: [0, 0, 0, 255],
      crosshair: false,
      copied: false,
    });
    session.boxes = vec![
      Rect::from_xywh(10.0, 10.0, 80.0, 60.0),
      Rect::from_xywh(35.0, 25.0, 30.0, 30.0),
    ];
    session.document.measurements.push(Measurement {
      id: 1,
      bounds: Rect::from_xywh(10.0, 10.0, 80.0, 60.0),
      label: ArtifactLabel::default(),
    });
    session.document.next_id = 1;
  }
  let (lines, objects) = state.center_aids();
  assert_eq!(lines.len(), 1);
  assert!(lines[0].x_accent);
  assert!(lines[0].y_accent);
  assert_eq!(objects.len(), 1);
  assert!(state.0.lock().unwrap().center_aid_cache.is_some());

  assert!(state.toggle_centerlines().is_some());
  assert_eq!(state.center_aids(), (Vec::new(), Vec::new()));
  assert!(state.toggle_centerlines().is_some());
  assert_eq!(state.center_aids(), (lines, objects));
}
