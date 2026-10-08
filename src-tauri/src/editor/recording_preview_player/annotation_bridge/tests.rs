// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn manager() -> PreviewPlayerManager {
  let layout = preview_layout(
    Some((
      1920,
      1080,
      crate::editor::recording_preview_player::layout::PreviewPaneKind::Screen,
    )),
    None,
    720,
  );
  let sources = PlayerSources {
    annotation_clips: Default::default(),
    scenes: Default::default(),
    audio_tracks: vec![],
    camera_duration_ms: None,
    camera_path: None,
    captures: Default::default(),
    composition_settings: None,
    cursor: None,
    #[cfg(target_os = "macos")]
    cursor_artworks: None,
    cursor_settings: Default::default(),
    keyboard: None,
    animation_ranges: Default::default(),
    keyboard_settings: Default::default(),
    duration_ms: 10_000,
    frames_per_second: Some(60.0),
    held_fills: [None, None],
    layout: layout.clone(),
    playback_layout: layout,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pins: None,
    playing: Default::default(),
    video_muted: Default::default(),
    preview_surface: None,
    noise: Default::default(),
    primary_kind: PrimaryRecordingKind::Screen,
    screen_path: "/tmp/annotation-test.mov".into(),
  };
  let mut manager = PreviewPlayerManager {
    sources: Some(sources),
    session_id: Some(7),
    position_ms: 2000,
    ..Default::default()
  };
  manager.annotation.pane = Some(0);
  manager.annotation.mode = 2;
  manager
}

#[test]
fn native_gesture_commits_once_and_cancel_restores_the_document() {
  let mut manager = manager();
  assert!(manager
    .annotation_gesture(
      SelectionGesturePhase::Begin,
      0,
      AnnotationGestureTarget::New,
      0.1,
      0.2,
      0,
      1920.0,
    )
    .is_none());
  let commit = manager
    .annotation_gesture(
      SelectionGesturePhase::End,
      0,
      AnnotationGestureTarget::New,
      0.6,
      0.7,
      0,
      1920.0,
    )
    .unwrap();
  assert_eq!(commit.session_id, 7);
  assert_eq!(commit.source_position_ms, 2000);
  assert_eq!(commit.annotations.len(), 1);
  assert_eq!(
    commit.selected_annotation_ids,
    vec![commit.annotations[0].id.clone()]
  );
  let before = manager
    .sources
    .as_ref()
    .unwrap()
    .annotation_clips
    .read()
    .unwrap()
    .clone();
  manager.annotation_gesture(
    SelectionGesturePhase::Begin,
    0,
    AnnotationGestureTarget::New,
    0.3,
    0.4,
    0,
    1920.0,
  );
  assert_eq!(
    manager
      .sources
      .as_ref()
      .unwrap()
      .annotation_clips
      .read()
      .unwrap()
      .len(),
    2
  );
  assert!(manager
    .annotation_gesture(
      SelectionGesturePhase::Cancel,
      0,
      AnnotationGestureTarget::New,
      0.5,
      0.6,
      0,
      1920.0,
    )
    .is_none());
  assert_eq!(
    *manager
      .sources
      .as_ref()
      .unwrap()
      .annotation_clips
      .read()
      .unwrap(),
    before
  );
  let selection = manager
    .annotation_gesture(
      SelectionGesturePhase::Begin,
      0,
      AnnotationGestureTarget::Select { index: 0 },
      0.1,
      0.2,
      0,
      1920.0,
    )
    .unwrap();
  assert_eq!(selection.annotations, commit.annotations);
  assert_eq!(
    *manager
      .sources
      .as_ref()
      .unwrap()
      .annotation_clips
      .read()
      .unwrap(),
    before
  );
}

#[test]
fn final_frame_drawing_uses_a_visible_source_instant() {
  let mut manager = manager();
  manager.position_ms = 10_000;
  manager.annotation_gesture(
    SelectionGesturePhase::Begin,
    0,
    AnnotationGestureTarget::New,
    0.1,
    0.2,
    0,
    1920.0,
  );
  let commit = manager
    .annotation_gesture(
      SelectionGesturePhase::End,
      0,
      AnnotationGestureTarget::New,
      0.6,
      0.7,
      0,
      1920.0,
    )
    .unwrap();
  assert_eq!(commit.source_position_ms, 9999);
  assert_eq!(manager.pane_annotations(0).len(), 1);
  manager.is_playing = true;
  assert!(manager
    .annotation_gesture(
      SelectionGesturePhase::Begin,
      0,
      AnnotationGestureTarget::New,
      0.1,
      0.2,
      0,
      1920.0,
    )
    .is_none());
}

#[test]
fn selects_a_screen_arrow_while_camera_is_the_active_layer() {
  let mut manager = manager();
  let annotation = crate::editor::annotations::arrow::new_arrow(
    "screen-arrow".into(),
    crate::editor::annotations::AnnotationPoint { x: 100.0, y: 100.0 },
    crate::editor::annotations::AnnotationPoint { x: 500.0, y: 100.0 },
    None,
  );
  manager
    .sources
    .as_ref()
    .unwrap()
    .annotation_clips
    .write()
    .unwrap()
    .push(RecordingAnnotationClip {
      path_ms: None,
      pin: None,
      annotation,
      track_id: AnnotationTrack::Primary,
      start_ms: 0,
      end_ms: 5000,
    });
  manager.annotation.pane = Some(1);
  manager.annotation.mode = 1;
  let commit = manager
    .annotation_gesture(
      SelectionGesturePhase::Begin,
      0,
      AnnotationGestureTarget::Select { index: 0 },
      0.1,
      0.1,
      0,
      1920.0,
    )
    .expect("one press must select an annotation on another video layer");
  assert_eq!(commit.selected_annotation_ids, vec!["screen-arrow"]);
  assert_eq!(manager.annotation.pane, Some(0));
  assert_eq!(manager.annotation_targets()[0].0, 0);
}

/// The Animate switch's last setting dresses the next arrow, the way the
/// style defaults do - except that it sits on the annotation rather than in its
/// style, so the bridge applies it instead of `new_arrow`.
#[test]
fn a_fresh_arrow_takes_the_remembered_animate_setting() {
  let mut manager = manager();
  manager.annotation.animated = Some(false);
  manager.annotation_gesture(
    SelectionGesturePhase::Begin,
    0,
    AnnotationGestureTarget::New,
    0.1,
    0.2,
    0,
    1920.0,
  );
  let commit = manager
    .annotation_gesture(
      SelectionGesturePhase::End,
      0,
      AnnotationGestureTarget::New,
      0.6,
      0.7,
      0,
      1920.0,
    )
    .unwrap();
  assert!(!commit.annotations[0].animated);
  assert!(
    !manager
      .sources
      .as_ref()
      .unwrap()
      .annotation_clips
      .read()
      .unwrap()[0]
      .annotation
      .animated
  );
}

/// A paste places its picture whatever tool is in hand, as a press with the
/// image tool in the pane's middle would; a drop places it where it
/// landed. Either is at its own size held inside the pane and kept in hand.
/// The tool the press borrowed is put back, and nothing is placed over a
/// playing recording.
#[test]
fn a_pasted_or_dropped_picture_lands_chosen_whatever_tool_is_in_hand() {
  use crate::editor::annotations::image::ImageArt;
  use crate::editor::annotations::AnnotationShape;
  use crate::editor::images::dropped::ImagePoint;

  let art = ImageArt {
    asset: "image:0123456789abcdef0123456789abcdef".to_owned(),
    aspect: 2.0,
    pixels: Some(4_000.0),
    play: None,
  };
  let placed = |commit: &gesture::Commit| {
    let Some(fresh) = commit.annotations.last() else {
      panic!("an image was not placed");
    };
    let AnnotationShape::Image {
      center,
      size,
      aspect,
      asset,
      ..
    } = &fresh.shape
    else {
      panic!("an image was not placed");
    };
    assert_eq!((asset.as_str(), *aspect), (art.asset.as_str(), art.aspect));
    assert_eq!(commit.selected_annotation_ids, vec![fresh.id.clone()]);
    (center.x, center.y, *size)
  };
  let mut manager = manager();
  manager.annotation.mode = 0;
  manager.is_playing = true;
  assert!(manager.place_image(art.clone(), None).is_none());
  manager.is_playing = false;
  let commit = manager.place_image(art.clone(), None).unwrap();
  assert_eq!(manager.annotation.mode, 0);
  assert!(manager.annotation.fresh.image.is_none());
  assert_eq!(placed(&commit), (960.0, 540.0, 1920.0));
  assert_eq!(manager.annotation.selected, commit.selected_annotation_ids);
  let dropped = ImagePoint {
    layer: 0,
    x: 0.25,
    y: 0.75,
  };
  let commit = manager.place_image(art.clone(), Some(dropped)).unwrap();
  assert_eq!(placed(&commit), (480.0, 810.0, 1920.0));
  assert_eq!(manager.annotation.selected, commit.selected_annotation_ids);
}

fn screen_arrow(id: &str, x: f64, start_ms: u64, end_ms: u64) -> RecordingAnnotationClip {
  RecordingAnnotationClip {
    path_ms: None,
    pin: None,
    annotation: crate::editor::annotations::arrow::new_arrow(
      id.into(),
      crate::editor::annotations::AnnotationPoint { x, y: 100.0 },
      crate::editor::annotations::AnnotationPoint {
        x: x + 200.0,
        y: 100.0,
      },
      None,
    ),
    track_id: AnnotationTrack::Primary,
    start_ms,
    end_ms,
  }
}

/// A manager with an arrow shown at the playhead and one that only arrives
/// later, both chosen, under the select tool.
fn grouped() -> PreviewPlayerManager {
  let mut manager = manager();
  *manager
    .sources
    .as_ref()
    .unwrap()
    .annotation_clips
    .write()
    .unwrap() = vec![
    screen_arrow("shown", 100.0, 0, 5000),
    screen_arrow("hidden", 900.0, 6000, 9000),
  ];
  manager.annotation.mode = 1;
  manager.annotation.selected = vec!["shown".into(), "hidden".into()];
  manager
}

fn start_x(annotation: &Annotation) -> f64 {
  match &annotation.shape {
    crate::editor::annotations::AnnotationShape::Arrow { start, .. } => start.x,
    _ => unreachable!(),
  }
}

fn clip_starts(manager: &PreviewPlayerManager) -> Vec<f64> {
  manager
    .sources
    .as_ref()
    .unwrap()
    .annotation_clips
    .read()
    .unwrap()
    .iter()
    .map(|clip| start_x(&clip.annotation))
    .collect()
}

fn carry(
  manager: &mut PreviewPlayerManager,
  phase: SelectionGesturePhase,
  x: f64,
) -> Option<gesture::Commit> {
  manager.annotation_gesture(phase, 0, AnnotationGestureTarget::Group, x, 0.1, 0, 1920.0)
}

#[test]
fn carrying_a_group_moves_the_members_hidden_at_the_playhead_in_one_commit() {
  let mut manager = grouped();
  assert!(carry(&mut manager, SelectionGesturePhase::Begin, 0.1).is_none());
  // A twentieth of the 1920-pixel source across: 96 pixels.
  let commit = carry(&mut manager, SelectionGesturePhase::End, 0.15).unwrap();
  let moved: Vec<_> = commit
    .annotations
    .iter()
    .map(|a| (a.id.as_str(), start_x(a)))
    .collect();
  assert_eq!(moved, vec![("shown", 196.0), ("hidden", 996.0)]);
  assert_eq!(clip_starts(&manager), vec![196.0, 996.0]);
  assert_eq!(commit.selected_annotation_ids, vec!["shown", "hidden"]);
}

#[test]
fn a_cancelled_group_carry_puts_every_member_back() {
  let mut manager = grouped();
  carry(&mut manager, SelectionGesturePhase::Begin, 0.1);
  carry(&mut manager, SelectionGesturePhase::Update, 0.3);
  assert_ne!(clip_starts(&manager), vec![100.0, 900.0]);
  carry(&mut manager, SelectionGesturePhase::Cancel, 0.3);
  assert_eq!(clip_starts(&manager), vec![100.0, 900.0]);
}

#[test]
fn a_toggle_adds_an_annotation_to_the_choice_and_takes_it_out() {
  let mut manager = grouped();
  manager.annotation.selected = vec!["hidden".into()];
  let toggle = |manager: &mut PreviewPlayerManager| {
    manager
      .annotation_gesture(
        SelectionGesturePhase::Begin,
        0,
        AnnotationGestureTarget::Toggle { index: 0 },
        0.1,
        0.1,
        0,
        1920.0,
      )
      .unwrap()
      .selected_annotation_ids
  };
  assert_eq!(toggle(&mut manager), vec!["hidden", "shown"]);
  assert_eq!(toggle(&mut manager), vec!["hidden"]);
}

/// A band over the whole picture chooses what the playhead shows there, and
/// leaves out what arrives later and what is pinned.
#[test]
fn a_marquee_chooses_what_it_touches_at_the_playhead_and_skips_pinned() {
  let mut manager = grouped();
  let mut pinned = screen_arrow("pinned", 500.0, 0, 5000);
  pinned.pin = Some(Default::default());
  manager
    .sources
    .as_ref()
    .unwrap()
    .annotation_clips
    .write()
    .unwrap()
    .push(pinned);
  manager.annotation.selected = Vec::new();
  let band = |manager: &mut PreviewPlayerManager, phase, x, y| {
    manager.annotation_gesture(phase, 0, AnnotationGestureTarget::Marquee, x, y, 0, 1920.0)
  };
  assert!(band(&mut manager, SelectionGesturePhase::Begin, 0.0, 0.0).is_none());
  let commit = band(&mut manager, SelectionGesturePhase::End, 1.0, 1.0).unwrap();
  assert_eq!(commit.selected_annotation_ids, vec!["shown"]);
}
