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
    composition_settings: None,
    cursor: None,
    cursor_artworks: None,
    cursor_settings: Default::default(),
    keyboard: None,
    keyboard_animation_ranges: Default::default(),
    keyboard_settings: Default::default(),
    duration_ms: 10_000,
    frames_per_second: Some(60.0),
    layout: layout.clone(),
    playback_layout: layout,
    playing: Default::default(),
    video_muted: Default::default(),
    preview_surface: None,
    primary_kind: PrimaryRecordingKind::Screen,
    screen_path: "/tmp/annotation-test.mov".into(),
  };
  let mut manager = PreviewPlayerManager::default();
  manager.sources = Some(sources);
  manager.session_id = Some(7);
  manager.position_ms = 2000;
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
      AnnotationGestureTarget::NewArrow,
      0.1,
      0.2
    )
    .is_none());
  let commit = manager
    .annotation_gesture(
      SelectionGesturePhase::End,
      0,
      AnnotationGestureTarget::NewArrow,
      0.6,
      0.7,
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
    AnnotationGestureTarget::NewArrow,
    0.3,
    0.4,
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
      AnnotationGestureTarget::NewArrow,
      0.5,
      0.6
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
    AnnotationGestureTarget::NewArrow,
    0.1,
    0.2,
  );
  let commit = manager
    .annotation_gesture(
      SelectionGesturePhase::End,
      0,
      AnnotationGestureTarget::NewArrow,
      0.6,
      0.7,
    )
    .unwrap();
  assert_eq!(commit.source_position_ms, 9999);
  assert_eq!(manager.annotation_marks(0).len(), 1);
  manager.is_playing = true;
  assert!(manager
    .annotation_gesture(
      SelectionGesturePhase::Begin,
      0,
      AnnotationGestureTarget::NewArrow,
      0.1,
      0.2
    )
    .is_none());
}

#[test]
fn selects_a_screen_arrow_while_camera_is_the_active_layer() {
  let mut manager = manager();
  let mark = crate::editor::annotations::model::new_arrow(
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
      annotation: mark,
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
    )
    .expect("one press must select a mark on another video layer");
  assert_eq!(
    commit.selected_annotation_id.as_deref(),
    Some("screen-arrow")
  );
  assert_eq!(manager.annotation.pane, Some(0));
  assert_eq!(manager.annotation_targets()[0].0, 0);
}
