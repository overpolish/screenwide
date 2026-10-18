// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The editor's workspace as every platform draws it: the retained scene its
//! layers are composed in, over the checkerboard. The platform surface keeps
//! the drawable, the window and the present.

pub mod backdrop;
pub mod editing;
pub mod scene;

/// The layer id of the keyboard shortcuts drawn over the picture.
pub const KEYBOARD_LAYER_ID: u32 = u32::MAX - 1;
/// The layer id of the captions drawn over the picture.
pub const CAPTIONS_LAYER_ID: u32 = u32::MAX - 2;

/// Whether `layer_id` is drawn over the picture rather than placed in it.
/// Every shortcut and caption takes its size and place from its tool, so a
/// press on one only reports it, for the editor to open that tool: it is
/// never selected, moved or resized.
pub const fn is_overlay_layer(layer_id: u32) -> bool {
  layer_id == KEYBOARD_LAYER_ID || layer_id == CAPTIONS_LAYER_ID
}

/// Annotations are authored in full-resolution source pixels; a decoded frame
/// may have its own grid, while stroke width follows the output resolution.
/// Moves `settings`' annotations from the `source` grid onto the `decoded`
/// one, their strokes from an `output_width` canvas onto `settings`' own.
pub fn remap_source(
  settings: &mut screenwide_output::ScreenshotOutputSettings,
  source: (u32, u32),
  decoded: (u32, u32),
  output_width: u32,
) {
  use screenwide_annotations::AnnotationPoint;
  let sx = f64::from(decoded.0) / f64::from(source.0.max(1));
  let sy = f64::from(decoded.1) / f64::from(source.1.max(1));
  let stroke = f64::from(settings.width) / f64::from(output_width.max(1));
  for annotation in &mut settings.annotations {
    annotation.shape = annotation.shape.mapped(|point| AnnotationPoint {
      x: point.x * sx,
      y: point.y * sy,
    });
    annotation.style.width *= stroke;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_annotations::{arrow::new_arrow, AnnotationPoint, AnnotationShape};

  #[test]
  fn a_proxy_decode_keeps_an_arrows_place_and_scales_its_stroke() {
    let mut output = screenwide_output::fixtures::test_output_settings(960, 540);
    output.annotations = vec![new_arrow(
      "a".into(),
      AnnotationPoint { x: 480.0, y: 270.0 },
      AnnotationPoint {
        x: 1440.0,
        y: 810.0,
      },
      None,
    )];
    let width = output.annotations[0].style.width;
    remap_source(&mut output, (1920, 1080), (960, 540), 1920);
    let AnnotationShape::Arrow { start, end, .. } = output.annotations[0].shape else {
      unreachable!()
    };
    assert_eq!((start.x, start.y), (240.0, 135.0));
    assert_eq!((end.x, end.y), (720.0, 405.0));
    assert_eq!(output.annotations[0].style.width, width / 2.0);
  }
}
