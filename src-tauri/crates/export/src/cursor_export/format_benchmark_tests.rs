// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Every format of a saved project exported one after another and then in
//! one pass from a single decode, timed, and each format's two files
//! compared. A project saved before formats existed gets the platforms'
//! sizes with its screen fitted and centred on each, standing in for the
//! editor's layout, which draws as much.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use super::project_benchmark_tests::field;
use crate::cursor_export::{export, export_formats, CursorExportRequest};
use crate::media_preview::{
  recording_info, BakedVideoExportOptions, ExportRunResult, VideoExportOptions,
};
use crate::track_selection::{AudioLayout, TrackSelection};
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::CameraOverlaySettings;

/// The main canvas, then TikTok, Shorts and Reels' 9:16, Instagram's
/// portrait and square, and YouTube's 1440p.
const FORMATS: &[(&str, u32, u32)] = &[
  ("main", 0, 0),
  ("tall", 1080, 1920),
  ("portrait", 1080, 1350),
  ("square", 1080, 1080),
  ("youtube", 2560, 1440),
];

/// `main`'s screen fitted, with a margin, and centred on a `width` by
/// `height` canvas; `main` itself for the main canvas.
fn laid_out(main: &ScreenshotOutputSettings, width: u32, height: u32) -> ScreenshotOutputSettings {
  let mut output = main.clone();
  if width == 0 {
    return output;
  }
  let scale =
    (f64::from(width) * 0.92 / main.crop_width).min(f64::from(height) * 0.92 / main.crop_height);
  output.crop_width = main.crop_width * scale;
  output.crop_height = main.crop_height * scale;
  output.crop_x = (f64::from(width) - output.crop_width) / 2.0;
  output.crop_y = (f64::from(height) - output.crop_height) / 2.0;
  output.image_x = output.crop_x + (main.image_x - main.crop_x) * scale;
  output.image_y = output.crop_y + (main.image_y - main.crop_y) * scale;
  output.image_width = main.image_width * scale;
  output.width = width;
  output.height = height;
  output
}

/// `edit` cut to its first `cap_ms` of output; its segments are shares of
/// the `duration_ms` recording.
fn capped(
  mut edit: screenwide_timeline_edit::RecordingTimelineEdit,
  cap_ms: f64,
  duration_ms: f64,
) -> screenwide_timeline_edit::RecordingTimelineEdit {
  let mut left = cap_ms;
  let mut kept = Vec::new();
  for mut segment in edit.segments.drain(..) {
    if left <= 0.0 {
      break;
    }
    let length = (segment.source_end - segment.source_start) * duration_ms / segment.playback_rate;
    if length > left {
      segment.source_end = segment.source_start + left * segment.playback_rate / duration_ms;
    }
    left -= length;
    kept.push(segment);
  }
  edit.segments = kept;
  edit
}

/// How alike two videos' pictures are, as FFmpeg's mean PSNR over every
/// frame: `inf` where they match exactly.
fn psnr(first: &Path, second: &Path) -> String {
  let output = std::process::Command::new(screenwide_sidecar::ffmpeg_path())
    .args(["-hide_banner", "-nostats", "-i"])
    .arg(first)
    .arg("-i")
    .arg(second)
    .args(["-lavfi", "psnr", "-f", "null", "-"])
    .output()
    .expect("FFmpeg runs");
  let log = String::from_utf8_lossy(&output.stderr);
  log
    .split("average:")
    .nth(1)
    .and_then(|rest| rest.split_whitespace().next())
    .unwrap_or("unknown")
    .to_owned()
}

/// What every format reads of the project: the same recording, cut the
/// same way.
struct Recording {
  screen: PathBuf,
  size: (u32, u32),
  duration_ms: u64,
  timeline: screenwide_timeline_edit::TimelinePlan,
  camera: Option<(PathBuf, (u32, u32), CameraOverlaySettings)>,
  cursor: Option<PathBuf>,
  cursor_effects: screenwide_cursor_effects::CursorEffectSettings,
  keyboard: Option<PathBuf>,
  keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings,
  cancelled: AtomicBool,
  selection: TrackSelection,
}

const VIDEO: VideoExportOptions = VideoExportOptions {
  compression: 2,
  resolution_scale_percent: 100,
  source_scale_percent: 100,
};

impl Recording {
  fn open(json: &serde_json::Value, directory: &Path) -> Self {
    let media = |key: &str| -> Option<PathBuf> {
      json
        .pointer(&format!("/recording/media/{key}"))
        .and_then(|value| value.as_str())
        .map(|relative| directory.join(relative))
    };
    let screen = media("primary").expect("the project has a screen recording");
    let screen_info = recording_info(&screen).expect("the screen recording opens");
    let duration_ms: u64 = field(json, "/recording/durationMs");
    let edit: screenwide_timeline_edit::RecordingTimelineEdit = field(json, "/timeline/edit");
    let edit = match std::env::var("SCREENWIDE_EXPORT_BENCH_SECONDS") {
      Ok(seconds) => capped(
        edit,
        seconds.parse::<f64>().unwrap() * 1_000.0,
        duration_ms as f64,
      ),
      Err(_) => edit,
    };
    let cursor_effects: screenwide_cursor_effects::CursorEffectSettings =
      field(json, "/look/look/cursorEffects");
    let keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings =
      field(json, "/look/look/keyboardEffects");
    let camera = media("camera")
      .filter(|_| field::<bool>(json, "/look/look/bakeCamera"))
      .map(|path| {
        let info = recording_info(&path).expect("the camera recording opens");
        let overlay = field(json, "/look/look/cameraOverlay");
        (path, (info.width, info.height), overlay)
      });
    Self {
      size: (screen_info.width, screen_info.height),
      screen,
      duration_ms,
      timeline: screenwide_timeline_edit::TimelinePlan::from_edit(&edit, duration_ms).unwrap(),
      camera,
      cursor: media("cursor").filter(|_| cursor_effects.bake),
      cursor_effects,
      keyboard: media("keyboard").filter(|_| keyboard_effects.bake),
      keyboard_effects,
      cancelled: AtomicBool::new(false),
      selection: TrackSelection::default(),
    }
  }

  fn request<'a>(
    &'a self,
    output: &'a ScreenshotOutputSettings,
    destination: &'a Path,
    on_progress: &'a mut dyn FnMut(u64),
  ) -> CursorExportRequest<'a> {
    CursorExportRequest {
      annotation_track: screenwide_annotations::timing::AnnotationTrack::Primary,
      audio_layout: AudioLayout::SeparateTracks,
      audio_source: None,
      camera: self
        .camera
        .as_ref()
        .map(|(path, (width, height), overlay)| {
          (
            path.as_path(),
            BakedVideoExportOptions {
              camera_drop_shadow: true,
              camera_height: *height,
              camera_width: *width,
              overlay: *overlay,
              screen_height: output.height,
              screen_width: output.width,
              video: VIDEO,
            },
          )
        }),
      camera_effects: None,
      camera_on_top: true,
      cancelled: &self.cancelled,
      captions: None,
      cursor: self.cursor.as_deref(),
      cursor_effects: self.cursor_effects,
      keyboard: self.keyboard.as_deref(),
      keyboard_effects: self.keyboard_effects,
      destination,
      duration_ms: self.duration_ms,
      height: self.size.1,
      on_progress,
      output,
      screen: &self.screen,
      selection: &self.selection,
      timeline: Some(&self.timeline),
      video: VIDEO,
      width: self.size.0,
    }
  }
}

fn destination(run: &str, name: &str) -> PathBuf {
  std::env::temp_dir().join(format!("screenwide-format-benchmark-{run}-{name}.mp4"))
}

#[test]
#[ignore = "set SCREENWIDE_EXPORT_BENCH_PROJECT to a .screenwide file"]
fn benchmarks_format_exports() {
  let Some(project) = std::env::var_os("SCREENWIDE_EXPORT_BENCH_PROJECT") else {
    eprintln!("skipped: set SCREENWIDE_EXPORT_BENCH_PROJECT to a .screenwide file");
    return;
  };
  let project = PathBuf::from(project);
  let json: serde_json::Value =
    serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
  let recording = Recording::open(&json, project.parent().unwrap());
  let main: ScreenshotOutputSettings = field(&json, "/look/look/recordingOutput/primary");
  let outputs = FORMATS
    .iter()
    .map(|&(_, width, height)| laid_out(&main, width, height))
    .collect::<Vec<_>>();
  let seconds = recording.timeline.duration_ms() as f64 / 1_000.0;

  // Opens the GPU and compiles its shaders, as an app's first export does.
  let warm = destination("warm", "main");
  export(recording.request(&outputs[0], &warm, &mut |_| {})).unwrap();
  let _ = std::fs::remove_file(&warm);

  let alone = FORMATS
    .iter()
    .map(|(name, _, _)| destination("alone", name))
    .collect::<Vec<_>>();
  let started = Instant::now();
  for (output, path) in outputs.iter().zip(&alone) {
    let result = export(recording.request(output, path, &mut |_| {})).unwrap();
    assert_eq!(result, ExportRunResult::Completed);
  }
  let one_after_another = started.elapsed().as_secs_f64();

  let together = FORMATS
    .iter()
    .map(|(name, _, _)| destination("together", name))
    .collect::<Vec<_>>();
  let mut progress = Vec::new();
  let mut reported = |processed_ms| progress.push(processed_ms);
  let mut quiet = (1..outputs.len()).map(|_| |_: u64| {}).collect::<Vec<_>>();
  let mut requests = vec![recording.request(&outputs[0], &together[0], &mut reported)];
  requests.extend(
    outputs[1..]
      .iter()
      .zip(&together[1..])
      .zip(&mut quiet)
      .map(|((output, path), quiet)| recording.request(output, path, quiet)),
  );
  let started = Instant::now();
  let result = export_formats(requests).unwrap();
  let one_pass = started.elapsed().as_secs_f64();
  assert_eq!(result, ExportRunResult::Completed);
  assert!(progress.windows(2).all(|pair| pair[0] <= pair[1]));
  assert_eq!(
    progress.last().copied(),
    Some(recording.timeline.duration_ms())
  );

  eprintln!(
    "[format-benchmark] {} formats of {seconds:.2}s of video: one after another {one_after_another:.2}s, one pass {one_pass:.2}s, {:.0}% less time",
    FORMATS.len(),
    100.0 * (1.0 - one_pass / one_after_another)
  );
  for ((name, _, _), (alone, together)) in FORMATS.iter().zip(alone.iter().zip(&together)) {
    let (first, second) = (
      recording_info(alone).unwrap(),
      recording_info(together).unwrap(),
    );
    assert_eq!((first.width, first.height), (second.width, second.height));
    eprintln!(
      "[format-benchmark] {name}: {}x{}, PSNR against its export alone {} dB",
      second.width,
      second.height,
      psnr(alone, together)
    );
    let _ = std::fs::remove_file(alone);
    let _ = std::fs::remove_file(together);
  }
}
