// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn mesh() -> Background {
  Background::Mesh {
    colors: vec![
      "#1B1F3B".to_owned(),
      "#4F46E5".to_owned(),
      "#9333EA".to_owned(),
      "#38BDF8".to_owned(),
    ],
    generator: crate::screenshots::default_mesh_generator(),
    locked_colors: vec![false; 4],
    points: vec![
      BackgroundMeshPoint {
        radius_x: 78.0,
        radius_y: 62.0,
        rotation: 18.0,
        x: 12.0,
        y: 14.0,
      },
      BackgroundMeshPoint {
        radius_x: 64.0,
        radius_y: 78.0,
        rotation: -34.0,
        x: 88.0,
        y: 22.0,
      },
      BackgroundMeshPoint {
        radius_x: 86.0,
        radius_y: 58.0,
        rotation: 62.0,
        x: 46.0,
        y: 96.0,
      },
    ],
    seed: 4211,
    warp_percent: 9.0,
  }
}

#[test]
fn names_a_file_per_background_and_size() {
  let solid = Background::Solid {
    color: "#FFFFFF".to_owned(),
  };
  let other = Background::Solid {
    color: "#000000".to_owned(),
  };
  assert_ne!(
    cache_key(&solid, 32, None).unwrap(),
    cache_key(&other, 32, None).unwrap()
  );
  assert_ne!(
    cache_key(&solid, 32, None).unwrap(),
    cache_key(&solid, 64, None).unwrap()
  );
  assert_eq!(
    cache_key(&solid, 32, None).unwrap(),
    cache_key(&solid, 32, None).unwrap()
  );
  // The file a swatch is drawn from is part of what names it, so the same
  // background drawn from the system's small copy is its own entry.
  assert_ne!(
    cache_key(&solid, 32, None).unwrap(),
    cache_key(&solid, 32, Some("/a/thumbnail.heic")).unwrap()
  );
}

#[test]
fn paints_a_flat_colour_across_the_swatch() {
  let canvas = render(
    &Background::Solid {
      color: "#112233".to_owned(),
    },
    16,
    None,
  )
  .unwrap();
  assert_eq!(canvas.dimensions(), (16, 16));
  assert_eq!(canvas.get_pixel(8, 8), &image::Rgba([17, 34, 51, 255]));
}

#[test]
fn falls_back_to_a_colour_when_the_picture_is_not_there() {
  let canvas = render(
    &Background::Image {
      path: "/no/such/wallpaper.png".to_owned(),
    },
    16,
    None,
  )
  .unwrap();
  assert_eq!(canvas.dimensions(), (16, 16));
}

#[test]
fn falls_back_to_a_colour_for_a_mesh_without_geometry() {
  let Background::Mesh {
    colors,
    generator,
    seed,
    warp_percent,
    ..
  } = mesh()
  else {
    unreachable!()
  };
  let canvas = render(
    &Background::Mesh {
      colors,
      generator,
      locked_colors: Vec::new(),
      points: Vec::new(),
      seed,
      warp_percent,
    },
    16,
    None,
  )
  .unwrap();
  assert_eq!(canvas.get_pixel(0, 0), &image::Rgba([27, 31, 59, 255]));
}

/// The tile stands for the wallpaper, which is a file the webview cannot
/// decode; the pixels come from the small copy beside it. This proves the
/// source is what is drawn, and that an unreadable one falls back rather
/// than failing.
#[test]
fn draws_a_picture_from_the_source_it_is_given() {
  let directory = std::env::temp_dir().join("screenwide-thumbnail-source-test");
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.png");
  image::RgbaImage::from_pixel(8, 8, image::Rgba([10, 200, 30, 255]))
    .save(&source)
    .unwrap();
  let background = Background::Image {
    path: "/no/such/wallpaper.heic".to_owned(),
  };
  let canvas = render(&background, 16, source.to_str()).unwrap();
  assert_eq!(canvas.get_pixel(8, 8), &image::Rgba([10, 200, 30, 255]));
  let fallback = render(&background, 16, Some("/no/such/source.png")).unwrap();
  assert_eq!(fallback.dimensions(), (16, 16));
  let _ = std::fs::remove_dir_all(&directory);
}
