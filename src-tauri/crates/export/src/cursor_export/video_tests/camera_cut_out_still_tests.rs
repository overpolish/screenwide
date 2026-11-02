// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A still of a recording, as the project's preview and scrub strip compose
//! it, draws a camera whose background is taken away as the export does: the
//! person casts the shadow from their outline, and the empty frame around
//! them casts none.

use super::camera_background_video_tests::{
  camera_with_matte, directory, frame_of, movie, output_settings, CAMERA, HEIGHT, VIDEO, WIDTH,
};
use super::*;
use crate::camera_effects::cpu::StillEffects;
use crate::camera_effects::CameraBackground;
use crate::decode::source_frame_image;
use screenwide_annotations::timing::AnnotationTrack;
use screenwide_compositor::compositor::still::{compose_still, StillCamera, StillInputs};
use screenwide_compositor::compositor::PictureLook;

/// The camera's 160 by 120 frame placed one pixel to one, Drop shadow on.
fn camera_options() -> BakedVideoExportOptions {
  BakedVideoExportOptions {
    camera_drop_shadow: true,
    camera_width: CAMERA.2,
    camera_height: CAMERA.3,
    screen_width: WIDTH,
    screen_height: HEIGHT,
    overlay: screenwide_scenes::CameraOverlaySettings {
      camera_width: f64::from(CAMERA.2),
      camera_x: f64::from(CAMERA.0 + CAMERA.2 / 2),
      camera_y: f64::from(CAMERA.1 + CAMERA.3 / 2),
      frame_width: f64::from(CAMERA.2),
      frame_height: f64::from(CAMERA.3),
      frame_x: f64::from(CAMERA.0),
      frame_y: f64::from(CAMERA.1),
      radius_percent: 0.0,
      follow_face: false,
    },
    video: VIDEO,
  }
}

#[test]
fn a_still_casts_a_cut_out_cameras_shadow_as_the_export_does() {
  let directory = directory("cut-out-still");
  let effects = camera_with_matte(&directory, CameraBackground::Transparent, 2_000);
  let live = effects.live.clone();
  let (screen_path, camera_path) = (directory.join("screen.mov"), directory.join("camera.mov"));
  let destination = directory.join("output.mp4");
  // A white screen, which the shadow darkens.
  movie(&screen_path, "drawbox=c=white:t=fill", (WIDTH, HEIGHT));
  let output = output_settings();
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    annotation_track: AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: Some((&camera_path, camera_options())),
    camera_effects: Some(effects),
    camera_on_top: true,
    cancelled: &cancelled,
    captions: None,
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 2_000,
    height: HEIGHT,
    on_progress: &mut |_| {},
    output: &output,
    screen: &screen_path,
    selection: &TrackSelection::default(),
    timeline: None,
    video: VIDEO,
    width: WIDTH,
  });
  assert_eq!(result.unwrap(), ExportRunResult::Completed);
  let exported = frame_of(&destination, "1.0");

  // The still at the same moment, drawn as `composed_frame.rs` draws it.
  let screen = source_frame_image(&screen_path, 1_000, 2_000).unwrap();
  let mut camera = source_frame_image(&camera_path, 1_000, 2_000).unwrap();
  let look = StillEffects::new(Some(live), 1_920)
    .unwrap()
    .apply(&mut camera, 1_000);
  assert!(look.cut_out, "the background is taken away at 1 s");
  let geometry = media_preview::bake_geometry(camera_options()).unwrap();
  let still = compose_still(
    &screen,
    &output,
    StillInputs {
      seconds: 1.0,
      cursor: None,
      camera: Some(StillCamera {
        image: &camera,
        crop: [
          geometry.crop_x,
          geometry.crop_y,
          geometry.crop_width,
          geometry.crop_height,
        ],
        frame: (
          geometry.frame_x,
          geometry.frame_y,
          geometry.frame_width,
          geometry.frame_height,
        ),
        radius: geometry.radius,
        drop_shadow: true,
        on_top: true,
        look,
      }),
      keyboard: None,
      foreground_only: false,
      look: PictureLook::default(),
      transparent_background: false,
    },
  )
  .unwrap();
  std::fs::remove_dir_all(directory).unwrap();
  let drawn = |x: u32, y: u32| {
    let at = ((y * still.width + x) * 4) as usize;
    [still.rgba[at], still.rgba[at + 1], still.rgba[at + 2]]
  };
  let darkest = |[r, g, b]: [u8; 3]| r.min(g).min(b);
  // The still matches the exported frame, up to what the encoder changes.
  let same = |what: &str, (x, y): (u32, u32)| {
    let (still, export) = (drawn(x, y), exported(x, y));
    assert!(
      still.iter().zip(export).all(|(a, b)| a.abs_diff(b) < 12),
      "{what} at ({x}, {y}): still {still:?}, export {export:?}"
    );
    still
  };

  let middle = CAMERA.1 + CAMERA.3 / 2;
  // The person, the left half of the frame, stays red. Not compared with the
  // export: H.264's 4:2:0 conversion dulls a saturated red by about 20.
  let person = drawn(CAMERA.0 + CAMERA.2 / 4, middle);
  assert!(person[0] > 240 && person[1] < 20, "the person: {person:?}");
  // Just right of their outline the screen shows, darkened by their shadow.
  let beside = same("beside the person", (CAMERA.0 + CAMERA.2 / 2 + 2, middle));
  assert!(
    darkest(beside) < 245,
    "the person casts a shadow: {beside:?}"
  );
  // Below the empty half of the frame there is no frame's shadow.
  let below = same(
    "below the empty frame",
    (CAMERA.0 + CAMERA.2 * 3 / 4, CAMERA.1 + CAMERA.3 + 2),
  );
  assert!(darkest(below) > 245, "no rectangular shadow: {below:?}");
}
