// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording scenes drawn offscreen through the compositor: each draws
//! a picture, and auto zoom's frames are the ones the editor's own path
//! makes of the same recording.

use std::io::Write;
use std::time::Duration;

use screenwide_compositor::compositor::cursor_placement::placed_cursor;
use screenwide_compositor::compositor::still::{compose_still, StillInputs};
use screenwide_compositor::compositor::PictureLook;
use screenwide_cursor_effects::{CursorCompositor, CursorEffectSettings};
use screenwide_imaging::CapturedImage;

use super::content::canvas_settings;
use super::script::cursor_records;
use super::{PictureBox, PointerKey, PointerScript, PreviewScene, SceneRenderer};

/// The screen's size in points, painted at twice that.
const POINTS: (f64, f64) = (320.0, 200.0);
const PIXELS: (u32, u32) = (640, 400);
const CANVAS: (u32, u32) = (480, 300);

/// A desktop: a gradient, with windows of their own colours on it, and
/// Screenwide's controls along the top where `with_screenwide`.
fn screen(with_screenwide: bool) -> CapturedImage {
  let (width, height) = PIXELS;
  let windows = [
    ([40, 40, 260, 220], [240, 240, 236]),
    ([340, 120, 600, 360], [60, 90, 160]),
  ];
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for y in 0..height {
    for x in 0..width {
      let mut colour = [
        (120 + x * 100 / width) as u8,
        (60 + y * 120 / height) as u8,
        150,
      ];
      for ([left, top, right, bottom], fill) in windows {
        if (left..right).contains(&x) && (top..bottom).contains(&y) {
          colour = fill;
        }
      }
      if with_screenwide && (250..390).contains(&x) && (8..40).contains(&y) {
        colour = [12, 12, 12];
      }
      rgba.extend_from_slice(&[colour[0], colour[1], colour[2], 255]);
    }
  }
  CapturedImage {
    width,
    height,
    rgba,
  }
}

fn key(at_ms: u32, (x, y): (f64, f64), pressed: bool) -> PointerKey {
  PointerKey {
    at_ms,
    x,
    y,
    pressed,
  }
}

/// Work in one window's corner, then in the other window.
fn script() -> PointerScript {
  let first = (70.0, 60.0);
  let second = (240.0, 130.0);
  PointerScript {
    duration_ms: 9_000,
    keys: vec![
      key(0, (160.0, 100.0), false),
      key(900, first, false),
      key(1_000, first, true),
      key(1_080, first, false),
      key(1_700, (78.0, 64.0), true),
      key(1_780, (78.0, 64.0), false),
      key(2_400, (66.0, 70.0), true),
      key(2_480, (66.0, 70.0), false),
      key(4_400, second, false),
      key(4_500, second, true),
      key(4_580, second, false),
      key(5_200, (232.0, 124.0), true),
      key(5_280, (232.0, 124.0), false),
      key(9_000, (232.0, 124.0), false),
    ],
    typing_ms: Vec::new(),
  }
}

/// A renderer drawing the fallback cursor artwork: a test process never
/// loads the system's, which takes the app's main thread.
fn renderer(scene: PreviewScene) -> SceneRenderer {
  let gpu = screenwide_gpu::shared().unwrap();
  let compositor = screenwide_compositor::compositor::Compositor::new(&gpu).unwrap();
  SceneRenderer::new(compositor, scene, vec![screen(false), screen(true)], POINTS).unwrap()
}

/// How many distinct colours the frame holds, a few of them enough to tell
/// a picture from an empty or flat canvas.
fn colours(frame: &CapturedImage) -> usize {
  let mut seen: Vec<[u8; 4]> = frame.rgba.as_chunks::<4>().0.to_vec();
  seen.sort_unstable();
  seen.dedup();
  seen.len()
}

fn every_scene() -> Vec<PreviewScene> {
  vec![
    PreviewScene::AutoZoom {
      on: true,
      script: script(),
    },
    PreviewScene::FrameRate {
      fps: 30,
      script: script(),
    },
    PreviewScene::IncludeScreenwide { included: true },
    PreviewScene::DesktopIcons { hidden: false },
    PreviewScene::SystemBars {
      kept: PictureBox {
        x: 0.0,
        y: 12.0,
        width: POINTS.0,
        height: POINTS.1 - 12.0,
      },
    },
  ]
}

#[test]
fn hiding_desktop_icons_records_the_bare_screen() {
  let frame = |scene: PreviewScene| {
    let renderer = renderer(scene);
    renderer
      .render(CANVAS, renderer.moment(Duration::ZERO))
      .unwrap()
      .rgba
  };
  let bare = frame(PreviewScene::IncludeScreenwide { included: false });
  let layered = frame(PreviewScene::IncludeScreenwide { included: true });
  assert_ne!(bare, layered);
  assert_eq!(frame(PreviewScene::DesktopIcons { hidden: true }), bare);
  assert_eq!(frame(PreviewScene::DesktopIcons { hidden: false }), layered);
}

#[test]
fn leaving_bars_out_records_only_the_kept_part_of_the_screen() {
  let frame = |scene: PreviewScene| {
    let renderer = renderer(scene);
    renderer
      .render(CANVAS, renderer.moment(Duration::ZERO))
      .unwrap()
  };
  let whole = PictureBox {
    x: 0.0,
    y: 0.0,
    width: POINTS.0,
    height: POINTS.1,
  };
  assert_eq!(
    frame(PreviewScene::SystemBars { kept: whole }).rgba,
    frame(PreviewScene::IncludeScreenwide { included: false }).rgba
  );
  // From the first window's corner, at the canvas's shape: the window fills
  // the canvas's start, and the gradient past its edge the rest.
  let kept = frame(PreviewScene::SystemBars {
    kept: PictureBox {
      x: 20.0,
      y: 20.0,
      width: 120.0,
      height: 75.0,
    },
  });
  // The window's colour, give or take the compositor's rounding.
  let is_window = |x: u32, y: u32| {
    let at = ((y * kept.width + x) * 4) as usize;
    [240_u8, 240, 236]
      .iter()
      .zip(&kept.rgba[at..at + 3])
      .all(|(want, got)| want.abs_diff(*got) <= 2)
  };
  assert!(is_window(2, 2));
  assert!(!is_window(CANVAS.0 - 4, 2));
}

#[test]
fn every_recording_scene_draws_a_picture() {
  for scene in every_scene() {
    let renderer = renderer(scene.clone());
    for elapsed in [0, 1_500, 3_000] {
      let frame = renderer
        .render(CANVAS, renderer.moment(Duration::from_millis(elapsed)))
        .unwrap();
      assert!(
        colours(&frame) > 16,
        "{scene:?} drew a flat frame at {elapsed}"
      );
    }
  }
}

#[test]
fn auto_zoom_zooms_into_the_work_and_back() {
  let zoomed = renderer(PreviewScene::AutoZoom {
    on: true,
    script: script(),
  });
  assert!(!zoomed.zooms().is_empty(), "the script made no zooms");
  let off = renderer(PreviewScene::AutoZoom {
    on: false,
    script: script(),
  });
  let inside = zoomed.zooms()[0].start_ms + 900;
  let moment = zoomed.moment(Duration::from_millis(inside));
  assert_ne!(
    zoomed.render(CANVAS, moment).unwrap().rgba,
    off.render(CANVAS, moment).unwrap().rgba,
    "the frame inside a zoom is drawn unzoomed"
  );
}

#[test]
fn frame_rate_holds_each_frame_for_its_length() {
  let thirty = renderer(PreviewScene::FrameRate {
    fps: 30,
    script: script(),
  });
  let sixty = renderer(PreviewScene::FrameRate {
    fps: 60,
    script: script(),
  });
  let at = |ms| Duration::from_millis(ms);
  assert_eq!(thirty.moment(at(1_040)), thirty.moment(at(1_060)));
  assert_ne!(sixty.moment(at(1_040)), sixty.moment(at(1_060)));
}

/// The editor's path for a recording on disk: the cursor track read from
/// its file, the zooms planned from that file, the frame arranged by the
/// scenes and composed as a still of the recording.
#[test]
fn auto_zoom_frames_are_the_editors() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-settings-preview-{}",
    std::process::id()
  ));
  std::fs::create_dir_all(&directory).unwrap();
  let path = directory.join("cursor.jsonl");
  let mut file = std::fs::File::create(&path).unwrap();
  for record in cursor_records(&script(), POINTS, PIXELS) {
    writeln!(file, "{}", serde_json::to_string(&record).unwrap()).unwrap();
  }
  drop(file);
  let duration_ms = u64::from(script().duration_ms);
  let zooms = screenwide_auto_zoom::plan(
    &path,
    None,
    duration_ms,
    screenwide_auto_zoom::VisibleArea::WHOLE,
  )
  .unwrap();
  let cursor = CursorCompositor::open(&path, None).unwrap();
  let preview = renderer(PreviewScene::AutoZoom {
    on: true,
    script: script(),
  });
  assert_eq!(preview.zooms(), zooms.as_slice());
  let picture = screen(false);
  for elapsed in [
    500,
    zooms[0].start_ms + 300,
    zooms[0].start_ms + 1_200,
    8_000,
  ] {
    let moment = preview.moment(Duration::from_millis(elapsed));
    let mut settings = canvas_settings(PIXELS, POINTS, CANVAS);
    screenwide_export::scenes::arrange_following(
      &zooms,
      &[],
      (moment.source_ms, moment.frame_ms),
      &mut settings,
      None,
      None,
    );
    let cursor = placed_cursor(
      &cursor,
      moment.source_ms,
      CursorEffectSettings::default(),
      PIXELS,
      &settings,
    )
    .unwrap();
    let editor = compose_still(
      &picture,
      &settings,
      StillInputs {
        seconds: moment.source_ms as f64 / 1_000.0,
        cursor,
        camera: None,
        keyboard: None,
        foreground_only: false,
        look: PictureLook::default(),
        transparent_background: false,
      },
    )
    .unwrap();
    let drawn = preview.render(CANVAS, moment).unwrap();
    let differing = drawn
      .rgba
      .iter()
      .zip(&editor.rgba)
      .filter(|(drawn, editor)| drawn.abs_diff(**editor) > 1)
      .count();
    assert_eq!(
      differing, 0,
      "the frame {elapsed} ms in differs from the editor's"
    );
  }
  let _ = std::fs::remove_dir_all(directory);
}

/// Every scene's frames as files, for a person to look at: run with
/// `SCREENWIDE_SETTINGS_PREVIEW_FRAMES` set to a folder.
#[test]
fn frames_for_review() {
  let Some(folder) = std::env::var_os("SCREENWIDE_SETTINGS_PREVIEW_FRAMES") else {
    return;
  };
  let folder = std::path::PathBuf::from(folder);
  std::fs::create_dir_all(&folder).unwrap();
  let mut scenes = every_scene();
  scenes.extend([
    PreviewScene::AutoZoom {
      on: false,
      script: script(),
    },
    PreviewScene::FrameRate {
      fps: 60,
      script: script(),
    },
    PreviewScene::IncludeScreenwide { included: false },
  ]);
  for (index, scene) in scenes.into_iter().enumerate() {
    let renderer = renderer(scene.clone());
    let name = format!("{index:02}-{}", kind(&scene));
    for elapsed in (0..9_000).step_by(750) {
      let frame = renderer
        .render(CANVAS, renderer.moment(Duration::from_millis(elapsed)))
        .unwrap();
      image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
        .unwrap()
        .save(folder.join(format!("{name}-{elapsed:05}.png")))
        .unwrap();
    }
  }
}

fn kind(scene: &PreviewScene) -> String {
  match scene {
    PreviewScene::AutoZoom { on, .. } => format!("auto-zoom-{}", if *on { "on" } else { "off" }),
    PreviewScene::FrameRate { fps, .. } => format!("frame-rate-{fps}"),
    PreviewScene::IncludeScreenwide { included } => format!("include-screenwide-{included}"),
    PreviewScene::DesktopIcons { hidden } => format!("desktop-icons-hidden-{hidden}"),
    PreviewScene::SystemBars { kept } => format!("system-bars-kept-{}", kept.height),
    _ => "tool".to_owned(),
  }
}
