// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A frame drawn by the slim variant keeping only the paths it takes looks as
//! it does drawn by the shader keeping every path. A path the CPU wrongly
//! leaves out shows here as a missing cursor, camera, background or turn.

use super::canvas_features::TEST_SLIM;
use super::render_test_helpers::{gpu, target, view};
use super::*;
use crate::annotation_gpu::prepared_arrows;
use screenwide_annotations::focus::model::new_focus;
use screenwide_annotations::outline::model::new_shape;
use screenwide_annotations::{Annotation, AnnotationPoint};
use screenwide_cursor_effects::GpuCursor;
use screenwide_imaging::CapturedImage;
use screenwide_keyboard_effects::KeyboardOverlay;
use screenwide_output::fixtures::test_output_settings;
use screenwide_output::CropPreviewRect;
use screenwide_scenes::{SceneMotion, SceneRotation};

/// What one frame draws besides its settings.
#[derive(Default)]
struct Frame {
  cursor: Option<GpuCursor>,
  camera: Option<bool>,
  magnifier: Option<CropMagnifier>,
  keyboard: Option<KeyboardOverlay>,
  annotations: Vec<Annotation>,
}

fn compositor() -> Compositor {
  Compositor::new(gpu()).expect("the preview compositor")
}

/// A grey screen in the middle of the canvas, with a shadow and rounding.
fn settings() -> ScreenshotOutputSettings {
  let mut settings = test_output_settings(1920, 1080);
  settings.drop_shadow = true;
  settings.radius_percent = 4.0;
  settings.background_radius_percent = 3.0;
  (settings.image_width, settings.crop_width) = (1200.0, 1200.0);
  settings.crop_height = 700.0;
  (settings.image_x, settings.crop_x) = (360.0, 360.0);
  (settings.image_y, settings.crop_y) = (190.0, 190.0);
  settings
}

fn cursor(blur: f32) -> GpuCursor {
  GpuCursor {
    opacity: 1.0,
    blur_delta_x: blur,
    blur_delta_y: 0.0,
    height: 48.0,
    hotspot_x: 0.0,
    hotspot_y: 0.0,
    rotation_radians: 0.0,
    scale: 1.0,
    style: 0,
    width: 32.0,
    x: 200.0,
    y: 150.0,
    clip_at_video_edge: false,
  }
}

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// The canvas, BGRA, drawn by the slim variant or by the full shader.
fn drawn(settings: &ScreenshotOutputSettings, frame: &Frame, slim: bool) -> Vec<u8> {
  TEST_SLIM.set(Some(slim));
  let compositor = compositor();
  let source = compositor
    .screenshot_source(&CapturedImage {
      width: 400,
      height: 300,
      rgba: (0..400 * 300)
        .flat_map(|index| [(index % 251) as u8, (index % 199) as u8, 90, 255])
        .collect(),
    })
    .expect("the source uploads");
  let camera = compositor
    .screenshot_source(&CapturedImage {
      width: 200,
      height: 200,
      rgba: [230, 40, 40, 255].repeat(200 * 200),
    })
    .expect("the camera uploads");
  let geometry = BakeGeometry {
    crop_height: 200,
    crop_width: 200,
    crop_x: 0,
    crop_y: 0,
    frame_height: 300,
    frame_width: 300,
    frame_x: 1450,
    frame_y: 700,
    output_height: 1080,
    output_width: 1920,
    radius: 40,
  };
  let prepared = prepared_arrows(&frame.annotations, (400, 300), settings, None, None, None)
    .expect("the annotations prepare");
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &source,
      settings,
      ComposedFrame {
        cursor: frame.cursor,
        keyboard: frame.keyboard,
        foreground_only: false,
        seconds: 1.5,
      },
      frame.camera.map(|on_top| (&camera, geometry, true, on_top)),
      frame.magnifier,
      &prepared,
    )
    .expect("the canvas draws");
  let pixels = gpu().read_texture(&output).expect("the target reads back");
  TEST_SLIM.set(None);
  pixels
}

fn assert_slim_draws_the_same(what: &str, settings: ScreenshotOutputSettings, frame: Frame) {
  let full = drawn(&settings, &frame, false);
  let slim = drawn(&settings, &frame, true);
  // Each variant is compiled on its own, and fast math may round a channel
  // apart by one; a path left out differs by far more.
  let worst = full
    .iter()
    .zip(&slim)
    .map(|(a, b)| a.abs_diff(*b))
    .max()
    .unwrap_or(0);
  assert!(
    worst <= 1,
    "{what}: the slim variant differs by up to {worst}"
  );
}

#[test]
#[ignore = "requires a GPU adapter"]
fn slim_variants_draw_what_the_full_shader_draws() {
  let with = |change: fn(&mut ScreenshotOutputSettings)| {
    let mut settings = settings();
    change(&mut settings);
    settings
  };
  assert_slim_draws_the_same("solid", settings(), Frame::default());
  assert_slim_draws_the_same(
    "cursor",
    settings(),
    Frame {
      cursor: Some(cursor(0.0)),
      ..Frame::default()
    },
  );
  assert_slim_draws_the_same(
    "moving cursor",
    settings(),
    Frame {
      cursor: Some(cursor(40.0)),
      ..Frame::default()
    },
  );
  assert_slim_draws_the_same(
    "classic mesh",
    with(|settings| {
      settings.background_type = "mesh".to_owned();
      settings.mesh_generator = "mesh".to_owned();
    }),
    Frame::default(),
  );
  assert_slim_draws_the_same(
    "generator",
    with(|settings| {
      settings.background_type = "mesh".to_owned();
      settings.mesh_generator = "aurora".to_owned();
      settings.mesh_colors.truncate(3);
    }),
    Frame::default(),
  );
  for on_top in [true, false] {
    assert_slim_draws_the_same(
      "camera",
      settings(),
      Frame {
        camera: Some(on_top),
        ..Frame::default()
      },
    );
  }
  assert_slim_draws_the_same(
    "turned",
    with(|settings| {
      settings.rotation = Some(SceneRotation {
        x: 10.0,
        y: 30.0,
        z: 0.0,
      });
    }),
    Frame::default(),
  );
  assert_slim_draws_the_same(
    "scene motion",
    with(|settings| {
      settings.scene_motion = Some(SceneMotion {
        screen: [0.9, 0.02, 0.01],
        image: [0.9, 0.02, 0.01],
        camera_frame: [0.7, 0.6, 0.15, 0.25],
        camera_crop: [0.0, 0.0, 1.0, 1.0],
        rotation: SceneRotation::default(),
        samples: 4,
      });
    }),
    Frame {
      camera: Some(true),
      ..Frame::default()
    },
  );
  assert_slim_draws_the_same(
    "crop preview",
    with(|settings| {
      settings.crop_preview = Some(CropPreviewRect {
        x: 500.0,
        y: 300.0,
        width: 600.0,
        height: 400.0,
      });
    }),
    Frame::default(),
  );
  assert_slim_draws_the_same(
    "magnifier",
    settings(),
    Frame {
      magnifier: Some(CropMagnifier {
        bounds: [0.0, 0.0, 1.0, 1.0],
        display_box: [0.0; 4],
        geometry: [900.0, 500.0, 160.0, 4.0],
        options: [0.0, 15.0, 0.0, 0.0],
      }),
      ..Frame::default()
    },
  );
  assert_slim_draws_the_same(
    "focus under an outline",
    settings(),
    Frame {
      cursor: Some(cursor(0.0)),
      annotations: vec![
        new_focus(
          "focus".to_owned(),
          [point(20.0, 20.0), point(200.0, 150.0)],
          None,
        ),
        new_shape(
          "outline".to_owned(),
          [point(250.0, 200.0), point(380.0, 280.0)],
          1,
          None,
        ),
      ],
      ..Frame::default()
    },
  );
  assert_slim_draws_the_same(
    "shortcut",
    settings(),
    Frame {
      keyboard: Some(shortcut()),
      ..Frame::default()
    },
  );
  let picture = std::env::temp_dir().join(format!(
    "screenwide-canvas-features-{}.png",
    std::process::id()
  ));
  let png = screenwide_output::encode_png(&CapturedImage {
    width: 64,
    height: 64,
    rgba: (0..64 * 64)
      .flat_map(|index| [(index % 64 * 4) as u8, (index / 64 * 4) as u8, 200, 255])
      .collect(),
  })
  .expect("the picture encodes");
  std::fs::write(&picture, png).expect("the picture saves");
  let mut with_picture = settings();
  with_picture.background_type = "image".to_owned();
  with_picture.background_image_path = Some(picture.to_string_lossy().into_owned());
  assert_slim_draws_the_same("background picture", with_picture, Frame::default());
  let _ = std::fs::remove_file(picture);
}

/// One key held down, fully shown.
fn shortcut() -> KeyboardOverlay {
  let mut keyboard = KeyboardOverlay {
    key_count: 1,
    animation: KeyboardOverlay::ANIMATION_POP,
    appearance: KeyboardOverlay::APPEARANCE_LIGHT,
    scale: 3.0,
    progress: 1.0,
    ..Default::default()
  };
  keyboard.keys[0].key_code = 55;
  keyboard.keys[0].visible = 1;
  keyboard.keys[0].progress = 1.0;
  keyboard.keys[0].alpha = 1.0;
  keyboard.keys[0].scale = 3.0;
  keyboard.keys[0].layout_progress = 1.0;
  keyboard.keys[0].layout_from_mask = 1;
  keyboard.keys[0].layout_to_mask = 1;
  keyboard
}
