// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An export of a saved project, timed, on the GPU an app would export on
//! but without its window: the compositor is made here rather than borrowed
//! from a preview. Captions and camera effects are left out, and the cursor
//! is drawn without artwork.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};

use super::export_surface::{ExportSurface, Held};
use crate::cursor_export::{export, CursorExportRequest};
use crate::media_preview::{
  recording_info, BakedVideoExportOptions, ExportRunResult, VideoExportOptions,
};
use crate::track_selection::{AudioLayout, TrackSelection};
use screenwide_compositor::compositor::{Compositor, NativeCursors};

struct Headless {
  compositor: Compositor,
  lock: Mutex<()>,
}

impl ExportSurface for Headless {
  fn compositor(&self) -> &Compositor {
    &self.compositor
  }

  fn gpu(&self) -> &'static screenwide_gpu::Gpu {
    screenwide_gpu::shared().expect("the GPU opens")
  }

  fn hold(&self) -> Result<Box<dyn Held + '_>, String> {
    Ok(Box::new(
      self
        .lock
        .lock()
        .map_err(|_| "The benchmark's compositor is poisoned")?,
    ))
  }
}

static SURFACE: LazyLock<Arc<Headless>> = LazyLock::new(|| {
  let gpu = screenwide_gpu::shared().expect("the GPU opens");
  let cursors = NativeCursors {
    size: (1, 1),
    layers: vec![0; 4 * 8],
    hotspots: [[0.0; 4]; 8],
  };
  let compositor = Compositor::new(gpu, cursors).expect("the compositor opens");
  compositor.compile_annotation_kinds_in_background();
  Arc::new(Headless {
    compositor,
    lock: Mutex::new(()),
  })
});

fn surface() -> Result<Arc<dyn ExportSurface>, String> {
  Ok(SURFACE.clone())
}

fn field<T: serde::de::DeserializeOwned>(project: &serde_json::Value, pointer: &str) -> T {
  let value = project
    .pointer(pointer)
    .unwrap_or_else(|| panic!("the project has no {pointer}"));
  serde_json::from_value(value.clone()).unwrap_or_else(|error| panic!("{pointer}: {error}"))
}

/// Exports the project twice and reports the second: the first opens the
/// GPU and leaves the driver its shaders to compile, as an app's launch does.
#[test]
#[ignore = "set SCREENWIDE_EXPORT_BENCH_PROJECT to a .screenwide file"]
fn benchmarks_project_export() {
  let project = PathBuf::from(std::env::var("SCREENWIDE_EXPORT_BENCH_PROJECT").unwrap());
  let directory = project.parent().unwrap().to_path_buf();
  let json: serde_json::Value =
    serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
  let media = |key: &str| -> Option<PathBuf> {
    json
      .pointer(&format!("/recording/media/{key}"))
      .and_then(|value| value.as_str())
      .map(|relative| directory.join(relative))
  };
  let screen = media("primary").expect("the project has a screen recording");
  let screen_info = recording_info(&screen).expect("the screen recording opens");
  let duration_ms: u64 = field(&json, "/recording/durationMs");
  let output: screenwide_output::ScreenshotOutputSettings =
    field(&json, "/look/look/recordingOutput/primary");
  let edit: screenwide_timeline_edit::RecordingTimelineEdit = field(&json, "/timeline/edit");
  let timeline = screenwide_timeline_edit::TimelinePlan::from_edit(&edit, duration_ms).unwrap();
  let cursor_effects: screenwide_cursor_effects::CursorEffectSettings =
    field(&json, "/look/look/cursorEffects");
  let keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings =
    field(&json, "/look/look/keyboardEffects");
  let video = VideoExportOptions {
    compression: 2,
    resolution_scale_percent: 100,
    source_scale_percent: 100,
  };
  let camera = media("camera")
    .filter(|_| field::<bool>(&json, "/look/look/bakeCamera"))
    .map(|path| {
      let info = recording_info(&path).expect("the camera recording opens");
      let options = BakedVideoExportOptions {
        camera_drop_shadow: true,
        camera_height: info.height,
        camera_width: info.width,
        overlay: field(&json, "/look/look/cameraOverlay"),
        screen_height: output.height,
        screen_width: output.width,
        video,
      };
      (path, options)
    });
  let cursor = media("cursor").filter(|_| cursor_effects.bake);
  let keyboard = media("keyboard").filter(|_| keyboard_effects.bake);
  let destination = std::env::temp_dir().join("screenwide-export-benchmark.mp4");
  for run in ["first", "timed"] {
    let cancelled = AtomicBool::new(false);
    let started = std::time::Instant::now();
    let result = export(CursorExportRequest {
      annotation_track: screenwide_annotations::timing::AnnotationTrack::Primary,
      audio_layout: AudioLayout::SeparateTracks,
      audio_source: None,
      camera: camera
        .as_ref()
        .map(|(path, options)| (path.as_path(), *options)),
      camera_effects: None,
      camera_on_top: true,
      cancelled: &cancelled,
      captions: None,
      cursor: cursor.as_deref(),
      cursor_effects,
      keyboard: keyboard.as_deref(),
      keyboard_effects,
      destination: &destination,
      duration_ms,
      height: screen_info.height,
      on_progress: &mut |_| {},
      output: &output,
      screen: &screen,
      selection: &TrackSelection::default(),
      surface,
      timeline: Some(&timeline),
      video,
      width: screen_info.width,
    })
    .unwrap();
    assert_eq!(result, ExportRunResult::Completed);
    let seconds = started.elapsed().as_secs_f64();
    eprintln!(
      "[export-benchmark] {run}: {:.2}s of video in {seconds:.2}s, {:.1}x realtime, to {}",
      timeline.duration_ms() as f64 / 1_000.0,
      timeline.duration_ms() as f64 / 1_000.0 / seconds,
      destination.display()
    );
  }
}
