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
    audio_tracks: vec![],
    camera_duration_ms: None,
    camera_path: None,
    capture_width_points: [0.0; 2],
    composition_settings: None,
    cursor: None,
    #[cfg(target_os = "macos")]
    cursor_artworks: None,
    cursor_settings: Default::default(),
    keyboard: None,
    keyboard_animation_ranges: Default::default(),
    keyboard_settings: Default::default(),
    duration_ms: 10_000,
    frames_per_second: Some(60.0),
    held_fills: None,
    layout: layout.clone(),
    playback_layout: layout,
    playing: Default::default(),
    video_muted: Default::default(),
    preview_surface: None,
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
    commit.selected_annotation_id.as_deref(),
    Some(commit.annotations[0].id.as_str())
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
  assert_eq!(
    commit.selected_annotation_id.as_deref(),
    Some("screen-arrow")
  );
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
