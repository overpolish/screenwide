// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing the annotations into a still that leaves the app as pixels.
//!
//! The clipboard has no layers, so a shot copied rather than opened carries
//! the arrows in its own pixels, and its highlights recolour those pixels.
//! They are drawn by the overlay's own pipeline, offscreen, and composed over
//! the still: the same shader draws the arrow whether it is on the desktop,
//! in the editor or on the clipboard.

use super::*;

use crate::editor::annotations::Annotation;
use crate::screenshots::CapturedImage;

/// The still with `annotations` drawn over it, in the still's own pixels.
///
/// The annotations arrive already placed in those pixels, so this is the
/// overlay's identity placement again, one drawn pixel per annotation pixel.
/// Their sizes are points of a capture taken at `scale`.
pub(crate) fn bake(
  image: &CapturedImage,
  annotations: &[Annotation],
  scale: f64,
) -> Result<CapturedImage, String> {
  if annotations.is_empty() || image.width == 0 || image.height == 0 {
    return Ok(image.clone());
  }
  let renderer = Renderer::new()?;
  let gpu = renderer.gpu();
  let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide annotate still"),
    size: wgpu::Extent3d {
      width: image.width,
      height: image.height,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: overlay_surface::FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  // A highlight recolours the still under it, so the still is its underlay,
  // and a blurring spotlight softens the same still.
  let still = underlay::upload(gpu, image)?;
  let blurs = annotations.iter().any(|annotation| {
    annotation.style.blur
      && matches!(
        annotation.shape,
        crate::editor::annotations::AnnotationShape::Spotlight { .. }
      )
  });
  let softened = if blurs {
    Some(underlay::upload(
      gpu,
      &crate::editor::annotations::spotlight::soften::soften(image),
    )?)
  } else {
    None
  };
  renderer.draw_arrows(
    &target.create_view(&Default::default()),
    (image.width, image.height),
    &arrows::placed_arrows(annotations, (0.0, 0.0), (1.0, 1.0), scale, None, None),
    Some(&still),
    softened.as_ref(),
  )?;
  let drawn = gpu
    .read_texture(&target)
    .map_err(|error| format!("The annotate still could not be read back: {error}"))?;
  let mut baked = image.clone();
  compose(&mut baked, &drawn);
  Ok(baked)
}

/// Source-over, in place. The drawn layer is premultiplied BGRA, rows tightly
/// packed at the still's size, and the still is opaque RGBA, so one
/// multiply-add per channel is the whole blend.
fn compose(image: &mut CapturedImage, drawn: &[u8]) {
  for (source, destination) in drawn.chunks_exact(4).zip(image.rgba.chunks_exact_mut(4)) {
    let alpha = u32::from(source[3]);
    if alpha == 0 {
      continue;
    }
    let over = |source: u8, destination: u8| {
      // The still is opaque, so rounding the remainder of the destination is
      // the only arithmetic the blend needs.
      let kept = u32::from(destination) * (255 - alpha) + 127;
      (u32::from(source) + kept / 255).min(255) as u8
    };
    destination[0] = over(source[2], destination[0]);
    destination[1] = over(source[1], destination[1]);
    destination[2] = over(source[0], destination[2]);
    destination[3] = 255;
  }
}

#[cfg(test)]
#[path = "bake_tests.rs"]
mod tests;
