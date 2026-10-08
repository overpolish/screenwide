// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Spotlights through the real video export: the shade over the frame's two
//! planes, and the blur over its source.

use super::*;
use crate::editor::annotations::spotlight::model::new_spotlight;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::AnnotationPoint;
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use std::path::{Path, PathBuf};
use std::process::Command;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
/// The light, in source pixels.
const LIGHT: [f64; 4] = [96.0, 48.0, 224.0, 132.0];

/// A two-second recording of ffmpeg's `pattern` source.
fn recording(directory: &Path, pattern: &str) -> PathBuf {
  let path = directory.join(format!("{pattern}.mov"));
  assert!(Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i"
    ])
    .arg(format!("{pattern}=s={WIDTH}x{HEIGHT}:r=10:d=2"))
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "8"])
    .arg(&path)
    .status()
    .unwrap()
    .success());
  path
}

fn clip(blur: bool) -> RecordingAnnotationClip {
  spotlight("spotlight", LIGHT, [0, 2_000], blur)
}

/// A spotlight over `light`, in source pixels, from `start_ms` to `end_ms`,
/// sharp-edged and square-cornered.
fn spotlight(
  id: &str,
  light: [f64; 4],
  [start_ms, end_ms]: [u64; 2],
  blur: bool,
) -> RecordingAnnotationClip {
  let point = |x, y| AnnotationPoint { x, y };
  let mut annotation = new_spotlight(
    id.to_owned(),
    [point(light[0], light[1]), point(light[2], light[3])],
    None,
  );
  annotation.style.softness = 0.0;
  annotation.style.radius = 0.0;
  annotation.style.blur = blur;
  RecordingAnnotationClip {
    path_ms: None,
    pin: None,
    annotation,
    track_id: AnnotationTrack::Primary,
    start_ms,
    end_ms,
  }
}

/// `source` exported one pixel to one, with `clips` over it.
fn exported(source: &Path, destination: &Path, clips: &[RecordingAnnotationClip]) -> PathBuf {
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      artifact_id: 1,
      next_segment_id: 1,
      keyboard_deletions: Box::default(),
      silence_cuts: Vec::new(),
      scene_clips: Vec::new(),
      annotation_clips: clips.to_vec(),
      segments: vec![RecordingTimelineSegment {
        id: 0,
        source_start: 0.0,
        source_end: 1.0,
        playback_rate: 1.0,
      }],
    },
    2_000,
  );
  let mut output = crate::screenshots::test_output_settings(WIDTH, HEIGHT);
  output.background_color = "#000000".into();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  output.crop_width = f64::from(WIDTH);
  output.crop_height = f64::from(HEIGHT);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = f64::from(WIDTH);
  output.image_x = 0.0;
  output.image_y = 0.0;
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
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
    destination,
    duration_ms: 2_000,
    height: HEIGHT,
    on_progress: &mut |_| {},
    output: &output,
    screen: source,
    selection: &TrackSelection::default(),
    timeline: timeline.as_ref(),
    video: VideoExportOptions {
      compression: 0,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
    width: WIDTH,
  });
  assert_eq!(result.unwrap(), ExportRunResult::Completed);
  destination.to_path_buf()
}

/// One frame of the movie at `path`, `time` seconds in, as RGB.
fn frame(path: &Path, time: &str) -> Vec<u8> {
  let output = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", time, "-i"])
    .arg(path)
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
  assert!(output.status.success());
  assert_eq!(output.stdout.len(), (WIDTH * HEIGHT * 3) as usize);
  output.stdout
}

/// The mean green channel over the rectangle `[x0, y0, x1, y1)`.
fn mean(image: &[u8], [x0, y0, x1, y1]: [u32; 4]) -> f64 {
  let mut total = 0.0;
  for y in y0..y1 {
    for x in x0..x1 {
      total += f64::from(image[((y * WIDTH + x) * 3 + 1) as usize]);
    }
  }
  total / f64::from((x1 - x0) * (y1 - y0))
}

/// How sharp the rectangle is: the mean step between neighbouring pixels.
fn detail(image: &[u8], [x0, y0, x1, y1]: [u32; 4]) -> f64 {
  let mut total = 0.0;
  for y in y0..y1 {
    for x in x0..x1 - 1 {
      let at = |x: u32| f64::from(image[((y * WIDTH + x) * 3 + 1) as usize]);
      total += (at(x + 1) - at(x)).abs();
    }
  }
  total / f64::from((x1 - x0 - 1) * (y1 - y0))
}

#[test]
fn an_exported_spotlight_shades_and_blurs_only_outside_its_light() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-spotlight-video-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let source = recording(&directory, "testsrc2");
  let plain = frame(&exported(&source, &directory.join("plain.mp4"), &[]), "1.0");
  let shaded = frame(
    &exported(&source, &directory.join("shaded.mp4"), &[clip(false)]),
    "1.0",
  );
  let blurred = frame(
    &exported(&source, &directory.join("blurred.mp4"), &[clip(true)]),
    "1.0",
  );
  let inside = [120, 64, 200, 116];
  let outside = [8, 8, 88, 40];
  // The light is left as it was; outside it is shaded by the fixed share.
  assert!((mean(&shaded, inside) - mean(&plain, inside)).abs() < 3.0);
  let ratio = mean(&shaded, outside) / mean(&plain, outside);
  assert!((ratio - 0.6).abs() < 0.05, "shaded to {ratio}");
  // Blur softens what is outside and nothing inside.
  assert!((detail(&blurred, inside) - detail(&plain, inside)).abs() < 1.5);
  assert!(
    detail(&blurred, outside) < detail(&shaded, outside) * 0.7,
    "{} against {}",
    detail(&blurred, outside),
    detail(&shaded, outside)
  );
  std::fs::remove_dir_all(directory).unwrap();
}

/// A two-second recording of one flat mid-grey, where a share of light reads
/// the same at every pixel. Sixty frames a second: an export keeps its
/// source's rate, and the glide's middle needs a frame of its own.
fn grey_recording(directory: &Path) -> PathBuf {
  let path = directory.join("grey.mov");
  assert!(Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i"
    ])
    .arg(format!("color=c=0x808080:s={WIDTH}x{HEIGHT}:r=60:d=2"))
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "8"])
    .arg(&path)
    .status()
    .unwrap()
    .success());
  path
}

/// Two spotlights butted up are one light moving on: the shade outside both
/// holds through the join rather than lifting as the first leaves and
/// falling again as the second arrives, and the light glides between them.
#[test]
fn butted_spotlights_hold_the_shade_through_the_join() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-spotlight-join-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let source = grey_recording(&directory);
  let plain = exported(&source, &directory.join("plain.mp4"), &[]);
  // The second ends short of the video's end: one that runs to it never
  // leaves, so its arrival may take longer than a third of its clip, and the
  // glide would run at its own pace rather than the third it is held to here.
  let joined = exported(
    &source,
    &directory.join("joined.mp4"),
    &[
      spotlight("first", LIGHT, [0, 1_000], false),
      spotlight(
        "second",
        [200.0, 100.0, 300.0, 170.0],
        [1_000, 1_900],
        false,
      ),
    ],
  );
  let outside = [8, 8, 88, 40];
  // Alone, the first would be all but gone a tenth of a second before its
  // end, and the second barely begun a tenth after its start.
  for time in ["0.9", "1.0", "1.1"] {
    let ratio = mean(&frame(&joined, time), outside) / mean(&frame(&plain, time), outside);
    assert!((ratio - 0.6).abs() < 0.05, "shaded to {ratio} at {time}s");
  }
  // The glide takes a third of the second's 900 ms. About halfway through,
  // the light lies over a stretch inside neither box; once the glide is done
  // that stretch is shaded again. The stretch is lit for the middle half of
  // the glide, so the frame read need not be its exact middle.
  let between = [236, 88, 244, 92];
  let ratio = |time| mean(&frame(&joined, time), between) / mean(&frame(&plain, time), between);
  assert!(
    ratio("1.13") > 0.95,
    "the light between them is at {}",
    ratio("1.13")
  );
  assert!(
    (ratio("1.4") - 0.6).abs() < 0.05,
    "shaded to {} after the glide",
    ratio("1.4")
  );
  std::fs::remove_dir_all(directory).unwrap();
}
