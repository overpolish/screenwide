// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use tauri::{LogicalPosition, LogicalSize};

use super::*;

fn region(x: f64, y: f64, width: f64, height: f64) -> Region {
  Region {
    position: LogicalPosition::new(x, y),
    size: LogicalSize::new(width, height),
  }
}

#[test]
fn passes_a_region_through_unchanged_at_one_times_scale() {
  let rect = physical_capture_rect(region(10.0, 20.0, 300.0, 200.0), 1.0, 1920, 1080).unwrap();
  assert_eq!(
    rect,
    CaptureRect {
      x: 10,
      y: 20,
      width: 300,
      height: 200
    }
  );
}

#[test]
fn doubles_a_region_on_a_retina_monitor() {
  let rect = physical_capture_rect(region(10.0, 20.0, 300.0, 200.0), 2.0, 3840, 2160).unwrap();
  assert_eq!(
    rect,
    CaptureRect {
      x: 20,
      y: 40,
      width: 600,
      height: 400
    }
  );
}

#[test]
fn rounds_the_edges_rather_than_the_size() {
  // Rounding the size independently would give 226 here, leaving the right
  // edge a pixel away from where the corner says it is.
  let rect = physical_capture_rect(region(10.4, 0.0, 150.3, 10.0), 1.5, 1920, 1080).unwrap();
  assert_eq!(rect.x, 16);
  assert_eq!(rect.width, 225);
  assert_eq!(rect.x + rect.width, 241);
}

#[test]
fn clamps_a_region_that_runs_past_the_monitor() {
  let rect = physical_capture_rect(region(1800.0, 1000.0, 400.0, 400.0), 1.0, 1920, 1080).unwrap();
  assert_eq!(
    rect,
    CaptureRect {
      x: 1800,
      y: 1000,
      width: 120,
      height: 80
    }
  );
}

#[test]
fn clamps_a_region_that_starts_before_the_monitor() {
  let rect = physical_capture_rect(region(-50.0, -30.0, 200.0, 100.0), 1.0, 1920, 1080).unwrap();
  assert_eq!(
    rect,
    CaptureRect {
      x: 0,
      y: 0,
      width: 150,
      height: 70
    }
  );
}

#[test]
fn fills_the_monitor_exactly_at_its_bounds() {
  let rect = physical_capture_rect(region(0.0, 0.0, 1920.0, 1080.0), 1.0, 1920, 1080).unwrap();
  assert_eq!(rect.width, 1920);
  assert_eq!(rect.height, 1080);
}

#[test]
fn rejects_a_region_entirely_off_the_monitor() {
  assert!(physical_capture_rect(region(2000.0, 0.0, 100.0, 100.0), 1.0, 1920, 1080).is_none());
}

#[test]
fn rejects_an_empty_or_nonsensical_region() {
  assert!(physical_capture_rect(region(0.0, 0.0, 0.0, 100.0), 1.0, 1920, 1080).is_none());
  assert!(physical_capture_rect(region(0.0, 0.0, 100.0, 100.0), 0.0, 1920, 1080).is_none());
  assert!(physical_capture_rect(region(f64::NAN, 0.0, 100.0, 100.0), 1.0, 1920, 1080).is_none());
}

#[test]
fn names_a_still_the_way_the_platform_does() {
  let captured_at = NaiveDate::from_ymd_opt(2026, 8, 8)
    .unwrap()
    .and_hms_opt(14, 32, 5)
    .unwrap();
  assert_eq!(
    capture_file_stem(captured_at),
    "Screenwide 2026-08-08 at 14.32.05"
  );
}

#[test]
fn zero_pads_every_field_of_the_name() {
  let captured_at = NaiveDate::from_ymd_opt(2026, 1, 2)
    .unwrap()
    .and_hms_opt(9, 5, 3)
    .unwrap();
  assert_eq!(
    capture_file_stem(captured_at),
    "Screenwide 2026-01-02 at 09.05.03"
  );
}

#[test]
fn uses_the_plain_name_when_nothing_is_in_the_way() {
  let path = unique_path(Path::new("/tmp"), "Shot", "png", &|_| false);
  assert_eq!(path, Path::new("/tmp/Shot.png"));
}

#[test]
fn counts_up_past_every_name_already_taken() {
  let taken: HashSet<PathBuf> = ["/tmp/Shot.png", "/tmp/Shot (2).png", "/tmp/Shot (3).png"]
    .iter()
    .map(PathBuf::from)
    .collect();
  let path = unique_path(Path::new("/tmp"), "Shot", "png", &|candidate| {
    taken.contains(candidate)
  });
  assert_eq!(path, Path::new("/tmp/Shot (4).png"));
}

#[test]
fn starts_the_suffix_at_two() {
  let taken: HashSet<PathBuf> = ["/tmp/Shot.png"].iter().map(PathBuf::from).collect();
  let path = unique_path(Path::new("/tmp"), "Shot", "png", &|candidate| {
    taken.contains(candidate)
  });
  assert_eq!(path, Path::new("/tmp/Shot (2).png"));
}

#[test]
fn deserializes_every_target_the_bar_can_send() {
  let screen: ScreenshotTarget =
    serde_json::from_str(r#"{"kind":"screen","monitorId":7}"#).unwrap();
  assert!(matches!(screen, ScreenshotTarget::Screen { monitor_id: 7 }));

  let window: ScreenshotTarget =
    serde_json::from_str(r#"{"kind":"window","windowId":42}"#).unwrap();
  assert!(matches!(window, ScreenshotTarget::Window { window_id: 42 }));

  let region: ScreenshotTarget = serde_json::from_str(
    r#"{"kind":"region","monitorId":7,"region":{"position":{"x":1,"y":2},"size":{"width":3,"height":4}}}"#,
  )
  .unwrap();
  let ScreenshotTarget::Region {
    monitor_id, region, ..
  } = region
  else {
    panic!("expected a region target");
  };
  assert_eq!(monitor_id, 7);
  assert_eq!(region.size.width, 3.0);

  let desktop_region: ScreenshotTarget = serde_json::from_str(
    r#"{"kind":"desktopRegion","monitorId":9,"region":{"position":{"x":-12,"y":5},"size":{"width":400,"height":200}}}"#,
  )
  .unwrap();
  let ScreenshotTarget::DesktopRegion {
    monitor_id, region, ..
  } = desktop_region
  else {
    panic!("expected a desktop region target");
  };
  assert_eq!(monitor_id, 9);
  assert_eq!(region.position.x, -12.0);
}

/// The crop tool shows the whole source, and the layer the crop will actually
/// produce over it.
///
/// The ghost underneath has to be flat - rounding and shadowing the uncropped
/// source would say the wrong thing about the result - so the canvas moves
/// both onto the second draw, and sizes the radius from the crop rectangle
/// rather than from the whole image.
#[cfg(target_os = "macos")]
#[test]
fn a_crop_preview_moves_the_rounding_and_the_shadow_onto_the_cropped_layer() {
  let mut settings = crate::screenshots::test_output_settings(400, 200);
  settings.drop_shadow = true;
  settings.radius_percent = 10.0;
  let plain = platform::native_canvas(2, 1, &settings, true).unwrap();
  assert!(plain.radius > 0);
  assert_eq!(plain.drop_shadow, 1);
  assert_eq!(plain.crop_preview, 0);

  settings.crop_preview = Some(output::CropPreviewRect {
    height: 80.0,
    width: 200.0,
    x: 50.0,
    y: 40.0,
  });
  let cropped = platform::native_canvas(2, 1, &settings, true).unwrap();
  assert_eq!(cropped.radius, 0);
  assert_eq!(cropped.drop_shadow, 0);
  assert_eq!(cropped.crop_preview, 1);
  assert_eq!(cropped.crop_preview_drop_shadow, 1);
  assert_eq!(cropped.crop_preview_x, 50.0);
  assert_eq!(cropped.crop_preview_y, 40.0);
  assert_eq!(cropped.crop_preview_width, 200.0);
  assert_eq!(cropped.crop_preview_height, 80.0);
  // Ten percent of the crop rectangle's shorter side, not of the whole image.
  assert_eq!(cropped.crop_preview_radius, 8.0);
}

/// The Metal canvas paints the same generators as the shared renderer.
///
/// The two are separate ports of the same reference shaders, so the only way
/// to know they agree is to render a canvas through each and compare. A
/// generator that read the wrong uniform, or a line that drifted while being
/// translated, shows up here rather than as a preview that does not match the
/// file it exports.
///
/// Both sides are asked for the same moment part way into a clip rather than
/// for the still at zero, so the animation is compared too: a generator whose
/// `time` reached one port's maths and not the other's would agree at zero and
/// only diverge once a recording started playing.
#[cfg(target_os = "macos")]
#[test]
fn the_native_canvas_paints_the_same_generators_as_the_shared_renderer() {
  const EDGE: u32 = 64;
  const SECONDS: f64 = 3.5;
  let colors = [
    "#0C1234".to_owned(),
    "#2882C8".to_owned(),
    "#DC78B4".to_owned(),
    "#FAE6BE".to_owned(),
  ];
  let source = CapturedImage {
    rgba: vec![255; 8],
    width: 2,
    height: 1,
  };
  for generator in mesh_generator::every_generator().iter().skip(1) {
    let mut settings = crate::screenshots::test_output_settings(EDGE, EDGE);
    settings.background_type = "mesh".to_owned();
    settings.mesh_generator = generator.name.to_owned();
    settings.mesh_colors = colors.to_vec();
    settings.mesh_seed = 4_242;
    // The screenshot is pushed into one corner so the rest of the canvas is
    // background and nothing else.
    settings.crop_width = 2.0;
    settings.crop_height = 1.0;
    settings.crop_x = 0.0;
    settings.crop_y = 0.0;
    settings.image_width = 2.0;
    settings.image_x = 0.0;
    settings.image_y = 0.0;
    let native = platform::compose_output_layers(
      &source, &settings, SECONDS, true, None, None, None, None, false, false,
    )
    .unwrap();
    let shared = mesh::mesh_canvas(
      EDGE,
      EDGE,
      generator.name,
      &colors,
      &[],
      4_242,
      0.0,
      SECONDS,
    )
    .unwrap();
    let mut worst = 0_i32;
    for y in 4..EDGE {
      for x in 4..EDGE {
        let offset = ((y * EDGE + x) * 4) as usize;
        for channel in 0..3 {
          let difference =
            i32::from(native.rgba[offset + channel]) - i32::from(shared.get_pixel(x, y).0[channel]);
          worst = worst.max(difference.abs());
        }
      }
    }
    // Both paths add a dither of up to one eight-bit step, and the two
    // backends round their own way, so a handful of steps apart is agreement
    // and anything more is a port that drifted.
    assert!(
      worst <= 6,
      "{} differs between the native canvas and the shared renderer by {worst}",
      generator.name
    );
  }
}

/// A procedural generator's swatch is the picture, not a flat colour.
///
/// The app's own mesh is drawn from blobs and has nothing to draw without
/// them, so a tile falls back to a colour there. A generator carries no blobs
/// at all, and reading that fallback the same way would leave every one of
/// its tiles a single flat tone.
#[test]
fn a_generator_tile_is_drawn_rather_than_filled() {
  let background = crate::settings::background_preset::Background::Mesh {
    colors: vec![
      "#0C1234".to_owned(),
      "#2882C8".to_owned(),
      "#DC78B4".to_owned(),
      "#FAE6BE".to_owned(),
    ],
    generator: "ribbons".to_owned(),
    locked_colors: Vec::new(),
    points: Vec::new(),
    seed: 4_242,
    warp_percent: 9.0,
  };
  let swatch = thumbnail::render(&background, 32, None).unwrap();
  let spread = |channel: usize| {
    let values = swatch.pixels().map(|pixel| pixel.0[channel]);
    values.clone().max().unwrap() - values.min().unwrap()
  };
  assert!(
    spread(0) > 16 || spread(1) > 16 || spread(2) > 16,
    "the tile was filled with one colour"
  );
}
