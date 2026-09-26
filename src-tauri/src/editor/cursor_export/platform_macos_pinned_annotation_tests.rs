// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::counter::model::new_counter;
use crate::editor::annotations::pin::model::PinKeyframe;
use crate::editor::annotations::pin::AnnotationPin;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::AnnotationPoint;
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use std::process::Command;

/// How fast the source scrolls up, in pixels a second.
const SCROLL: f64 = 40.0;

/// The middle of the yellow disc in the exported frame `time` seconds in, or
/// `None` where no disc is drawn.
fn disc_centre(destination: &std::path::Path, time: &str, width: u32) -> Option<(f64, f64)> {
  let frame = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", time, "-i"])
    .arg(destination)
    .args([
      "-frames:v",
      "1",
      "-f",
      "rawvideo",
      "-pix_fmt",
      "rgb24",
      "pipe:1",
    ])
    .output()
    .unwrap();
  assert!(frame.status.success());
  let (mut count, mut x, mut y) = (0.0, 0.0, 0.0);
  for (index, rgb) in frame.stdout.as_chunks::<3>().0.iter().enumerate() {
    if rgb[0] > 180 && rgb[1] > 150 && rgb[2] < 90 {
      count += 1.0;
      x += (index % width as usize) as f64;
      y += (index / width as usize) as f64;
    }
  }
  (count > 30.0).then(|| (x / count, y / count))
}

/// A counter pinned to a scrolling page is exported where the page carried
/// it, frame by frame, through the whole pipeline: the recording decoded for
/// tracking, the path worked out, and the Metal export drawing the records
/// it becomes.
#[test]
fn exports_a_pinned_counter_where_the_scrolling_content_carried_it() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-pinned-counter-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let destination = directory.join("output.mp4");
  // A still page of grey noise, taller than the frame, scrolled up steadily.
  let scroll = format!("crop=320:180:0:'t*{SCROLL}'");
  assert!(Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "color=c=gray:s=320x400:r=30:d=2,noise=alls=90:allf=u:all_seed=7,gblur=sigma=0.8",
      "-vf",
      &scroll,
      "-c:v",
      "libx264",
      "-crf",
      "12",
      "-g",
      "15",
      "-pix_fmt",
      "yuv420p",
    ])
    .arg(&source)
    .status()
    .unwrap()
    .success());
  let mut counter = new_counter(
    "pinned-counter".into(),
    AnnotationPoint { x: 160.0, y: 120.0 },
    1,
    None,
    None,
  );
  counter.animated = false;
  counter.style.color = "#ffcc00".into();
  counter.style.width = 28.0;
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      artifact_id: 1,
      next_segment_id: 1,
      keyboard_deletions: Box::default(),
      annotation_clips: vec![RecordingAnnotationClip {
        annotation: counter,
        track_id: AnnotationTrack::Primary,
        start_ms: 0,
        end_ms: 2_000,
        pin: Some(AnnotationPin {
          pinned_ms: 0,
          keyframes: vec![PinKeyframe {
            ms: 0,
            dx: 0.0,
            dy: 0.0,
          }],
          path: None,
        }),
      }],
      segments: vec![RecordingTimelineSegment {
        id: 0,
        source_start: 0.0,
        source_end: 1.0,
        playback_rate: 1.0,
      }],
    },
    2_000,
  )
  .unwrap();
  let mut output = crate::screenshots::test_output_settings(320, 180);
  output.background_color = "#000000".into();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  output.crop_width = 320.0;
  output.crop_height = 180.0;
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = 320.0;
  output.image_x = 0.0;
  output.image_y = 0.0;
  let cancelled = AtomicBool::new(false);
  assert_eq!(
    export(CursorExportRequest {
      annotation_track: AnnotationTrack::Primary,
      audio_layout: AudioLayout::SeparateTracks,
      audio_source: None,
      camera: None,
      camera_on_top: true,
      cancelled: &cancelled,
      cursor: None,
      cursor_effects: CursorEffectSettings::default(),
      keyboard: None,
      keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
      destination: &destination,
      duration_ms: 2_000,
      height: 180,
      on_progress: &mut |_| {},
      output: &output,
      screen: &source,
      selection: &TrackSelection::default(),
      timeline: Some(&timeline),
      video: VideoExportOptions {
        compression: 0,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
      width: 320,
    })
    .unwrap(),
    ExportRunResult::Completed
  );
  let (start_x, start_y) = disc_centre(&destination, "0.0", 320).expect("the counter at the start");
  for time in [0.5, 1.0, 1.5] {
    let (x, y) = disc_centre(&destination, &time.to_string(), 320).expect("the counter");
    let expected = start_y - SCROLL * time;
    assert!(
      (y - expected).abs() < 2.0 && (x - start_x).abs() < 2.0,
      "at {time}s the counter is at {x:.1}, {y:.1}; the page put it at {start_x:.1}, {expected:.1}"
    );
  }
  std::fs::remove_dir_all(directory).unwrap();
}
