// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing live annotations into a still that leaves the app as pixels.
//!
//! The clipboard has no layers, so the annotations a shot covered are
//! composited into it here, by the same compositor the editor exports with,
//! over an identity canvas: the image at its own size, no padding, no
//! background, no rounding.

use crate::editor::annotations::Annotation;

use super::{
  mesh::MeshGradientPoint, source_crop::NormalizedSourceRect, CapturedImage,
  ScreenshotOutputSettings,
};

fn identity_settings(
  image: &CapturedImage,
  annotations: Vec<Annotation>,
) -> ScreenshotOutputSettings {
  let width = f64::from(image.width);
  let height = f64::from(image.height);
  ScreenshotOutputSettings {
    annotations,
    background_color: "#000000".to_owned(),
    background_image_path: None,
    background_type: "solid".to_owned(),
    background_radius_percent: 0.0,
    crop_height: height,
    crop_width: width,
    crop_preview: None,
    crop_x: 0.0,
    crop_y: 0.0,
    drop_shadow: false,
    height: image.height,
    image_width: width,
    image_x: 0.0,
    image_y: 0.0,
    legacy_mode: None,
    // Never painted on a solid canvas, but the compositor's colour table is
    // filled from these whatever the background is.
    mesh_colors: vec!["#000000".to_owned(); 5],
    mesh_generator: super::default_mesh_generator(),
    mesh_locked_colors: vec![false; 5],
    mesh_points: Vec::<MeshGradientPoint>::new(),
    mesh_seed: 0,
    mesh_warp_percent: 0.0,
    radius_percent: 0.0,
    recenter_inset_color: None,
    source_crop: NormalizedSourceRect {
      height: 1.0,
      width: 1.0,
      x: 0.0,
      y: 0.0,
    },
    width: image.width,
  }
}

/// The still with `annotations` drawn over it. A shot too small for the
/// compositor's canvas floor is returned as it was: nothing is lost that the
/// editor could not still show.
pub(crate) fn bake_annotations(
  image: &CapturedImage,
  annotations: Vec<Annotation>,
) -> Result<CapturedImage, String> {
  if annotations.is_empty() {
    return Ok(image.clone());
  }
  let settings = identity_settings(image, annotations);
  if super::output::output_dimensions(&settings).is_err() {
    return Ok(image.clone());
  }
  super::platform::compose_output_layers(
    image, &settings, 0.0, true, None, None, None, None, false, false,
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::{
    AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
  };

  fn arrow(width: f64) -> Annotation {
    Annotation {
      above_camera: false,
      animated: true,
      id: "arrow".to_owned(),
      reveal: Default::default(),
      shape: AnnotationShape::Arrow {
        start: AnnotationPoint { x: 20.0, y: 100.0 },
        control: AnnotationPoint { x: 100.0, y: 100.0 },
        end: AnnotationPoint { x: 180.0, y: 100.0 },
      },
      style: AnnotationStyle {
        align: Default::default(),
        color: "#ff0000".to_owned(),
        head: AnnotationHead::default(),
        width,
      },
    }
  }

  fn black(width: u32, height: u32) -> CapturedImage {
    CapturedImage {
      rgba: [0, 0, 0, 255].repeat(width as usize * height as usize),
      width,
      height,
    }
  }

  fn pixel(image: &CapturedImage, x: u32, y: u32) -> &[u8] {
    let offset = ((y * image.width + x) * 4) as usize;
    &image.rgba[offset..offset + 4]
  }

  #[test]
  fn the_arrow_lands_in_the_still_at_its_own_pixels() {
    let baked = bake_annotations(&black(200, 200), vec![arrow(8.0)]).unwrap();

    assert_eq!((baked.width, baked.height), (200, 200));
    assert!(pixel(&baked, 100, 100)[0] > 200, "the shaft is red");
    assert_eq!(
      pixel(&baked, 100, 20),
      [0, 0, 0, 255],
      "away from it is untouched"
    );
  }

  #[test]
  fn a_shot_under_the_canvas_floor_is_returned_as_it_was() {
    let source = black(32, 32);
    let baked = bake_annotations(&source, vec![arrow(2.0)]).unwrap();

    assert_eq!(baked.rgba, source.rgba);
  }
}
