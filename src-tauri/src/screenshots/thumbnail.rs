// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A background as a swatch, painted by the code that paints it for real.
//!
//! A tile used to be a CSS likeness of a background: close for a flat colour,
//! a guess for a mesh, and nothing at all for a picture at the size a grid
//! shows it. This renders the background itself at the tile's size and hands
//! the webview a file, so what is offered in the grid is what the canvas gets.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::settings::background_preset::{Background, BackgroundMeshPoint};

use super::mesh::{mesh_canvas, MeshGradientPoint};
use super::mesh_generator::DEFAULT_GENERATOR;
use super::{background_image_swatch, encode_png, parse_hex_colour, CapturedImage};

/// The sizes a tile is ever asked for, held to something a swatch can use: a
/// thumbnail is a picture of a background, not a second export path.
const MINIMUM_SIZE: u32 = 8;
const MAXIMUM_SIZE: u32 = 512;

/// Where the rendered swatches live. One file per background and size, named
/// by a hash of both, so a background already drawn is never drawn again.
fn cache_directory(app: &AppHandle) -> Result<PathBuf, String> {
  let directory = app
    .path()
    .app_cache_dir()
    .map_err(|error| format!("The thumbnail cache directory is not available: {error}"))?
    .join("background-thumbnails");
  std::fs::create_dir_all(&directory)
    .map_err(|error| format!("The thumbnail cache directory could not be made: {error}"))?;
  Ok(directory)
}

/// A background, a size, and the file it is drawn from as one name. The JSON
/// is the background's own wire form, so two presets that paint the same
/// picture share a file.
fn cache_key(background: &Background, size: u32, source: Option<&str>) -> Result<String, String> {
  let description = serde_json::to_string(background)
    .map_err(|error| format!("The background could not be described: {error}"))?;
  let mut hasher = DefaultHasher::new();
  description.hash(&mut hasher);
  size.hash(&mut hasher);
  source.hash(&mut hasher);
  Ok(format!("{:016x}-{size}", hasher.finish()))
}

fn point(source: &BackgroundMeshPoint) -> MeshGradientPoint {
  MeshGradientPoint {
    radius_x: source.radius_x,
    radius_y: source.radius_y,
    rotation: source.rotation,
    x: source.x,
    y: source.y,
  }
}

fn solid_canvas(color: &str, size: u32) -> Result<image::RgbaImage, String> {
  Ok(image::RgbaImage::from_pixel(
    size,
    size,
    image::Rgba(parse_hex_colour(color)?),
  ))
}

/// The background at a swatch's size.
///
/// `source` is a second file to draw from, where the caller has a smaller
/// copy of the same picture: the system keeps swatch-sized copies of its own
/// desktop pictures. It is a source for the pixels only, and one that cannot
/// be read leaves the background's own file to be drawn.
///
/// A picture that will not load and a mesh saved without its geometry both
/// fall back to a flat colour rather than to an error, the way an export
/// does: a tile must still be drawable when the file behind it has moved.
pub(super) fn render(
  background: &Background,
  size: u32,
  source: Option<&str>,
) -> Result<image::RgbaImage, String> {
  match background {
    Background::Image { path } => {
      let picture = source
        .and_then(|source| background_image_swatch(source, size, size))
        .or_else(|| background_image_swatch(path, size, size));
      match picture {
        Some(picture) => Ok(picture),
        None => solid_canvas("#171717", size),
      }
    }
    Background::Solid { color } => solid_canvas(color, size),
    Background::Mesh {
      colors,
      generator,
      points,
      seed,
      warp_percent,
      ..
    } => {
      let points = points.iter().map(point).collect::<Vec<_>>();
      let fallback = colors.first().map_or("#171717", String::as_str);
      // Only the app's own mesh is drawn from blobs, so only it has nothing
      // to draw without them.
      if points.is_empty() && generator.as_str() == DEFAULT_GENERATOR {
        return solid_canvas(fallback, size);
      }
      // A swatch is a still, so every generator is drawn at time zero.
      let (seed, warp, still) = (*seed, *warp_percent, 0.0);
      mesh_canvas(size, size, generator, colors, &points, seed, warp, still)
        .or_else(|_| solid_canvas(fallback, size))
    }
  }
}

/// Renders one background at `size` by `size` and answers the file it was
/// written to, which the webview reads through the asset protocol.
///
/// The work is done off the main thread: the mesh goes through the same GPU
/// renderer an export uses, and a grid of tiles asks for several at once.
#[tauri::command]
pub async fn render_background_thumbnail(
  app: AppHandle,
  background: Background,
  size: u32,
  source_path: Option<String>,
) -> Result<String, String> {
  let size = size.clamp(MINIMUM_SIZE, MAXIMUM_SIZE);
  let key = cache_key(&background, size, source_path.as_deref())?;
  let path = cache_directory(&app)?.join(format!("{key}.png"));
  if path.is_file() {
    return Ok(path.to_string_lossy().into_owned());
  }
  let destination = path.clone();
  tauri::async_runtime::spawn_blocking(move || {
    let canvas = render(&background, size, source_path.as_deref())?;
    let png = encode_png(&CapturedImage {
      height: canvas.height(),
      width: canvas.width(),
      rgba: canvas.into_raw(),
    })?;
    std::fs::write(&destination, png)
      .map_err(|error| format!("The background thumbnail could not be written: {error}"))
  })
  .await
  .map_err(|error| error.to_string())??;
  Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
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
}
