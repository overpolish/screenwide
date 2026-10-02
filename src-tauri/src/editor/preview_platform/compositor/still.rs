// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One canvas composed into pixels: an exported or copied screenshot, a layer
//! of the screenshot editor, a still frame of a recording. The canvas is
//! drawn offscreen by the compositor and read back.

use std::sync::{LazyLock, Mutex};

use super::*;
use crate::editor::cursor_effects::GpuCursor;
use crate::screenshots::CapturedImage;

/// The camera bubble a still carries: its frame, the part of it shown, and
/// where that sits on the canvas, in output pixels.
pub(crate) struct StillCamera<'a> {
  pub(crate) image: &'a CapturedImage,
  /// x, y, width and height in the camera frame.
  pub(crate) crop: [u32; 4],
  /// x, y, width and height on the canvas.
  pub(crate) frame: (i32, i32, u32, u32),
  pub(crate) radius: u32,
  pub(crate) drop_shadow: bool,
  pub(crate) on_top: bool,
}

/// What one still draws besides the picture and its annotations.
pub(crate) struct StillInputs<'a> {
  pub(crate) seconds: f64,
  /// The cursor, and the artwork its styles are drawn with.
  pub(crate) cursor: Option<(GpuCursor, &'a [CursorArtwork<'a>])>,
  pub(crate) camera: Option<StillCamera<'a>>,
  pub(crate) keyboard: Option<KeyboardOverlay>,
  pub(crate) foreground_only: bool,
  /// Whether the canvas outside its rounded corners stays transparent, or is
  /// opaque black as an encoder that has no alpha would see it.
  pub(crate) transparent_background: bool,
}

/// Every still is composed through one compositor, one at a time: its
/// uniforms and lists are written for each draw before it is submitted.
static STILLS: LazyLock<Result<Mutex<Compositor>, String>> =
  LazyLock::new(|| Compositor::new(crate::gpu::shared()?, NativeCursors::none()).map(Mutex::new));

/// `image` composed on the canvas `settings` describe, as RGBA whose colour
/// is premultiplied by its alpha.
pub(crate) fn compose_still(
  image: &CapturedImage,
  settings: &ScreenshotOutputSettings,
  inputs: StillInputs<'_>,
) -> Result<CapturedImage, String> {
  let mut compositor = STILLS
    .as_ref()
    .map_err(Clone::clone)?
    .lock()
    .map_err(|_| "The still compositor is unavailable".to_owned())?;
  if let Some((_, artworks)) = inputs.cursor {
    compositor.use_cursor_artworks(artworks)?;
  }
  let gpu = compositor.gpu();
  let (width, height) = crate::screenshots::output_dimensions(settings)?;
  let mut source = compositor.screenshot_source(image)?;
  source.picture = Some(std::sync::Arc::new(image.clone()));
  let camera = inputs
    .camera
    .as_ref()
    .map(|camera| {
      Ok::<_, String>((
        compositor.screenshot_source(camera.image)?,
        BakeGeometry {
          crop_x: camera.crop[0],
          crop_y: camera.crop[1],
          crop_width: camera.crop[2],
          crop_height: camera.crop[3],
          frame_x: camera.frame.0,
          frame_y: camera.frame.1,
          frame_width: camera.frame.2,
          frame_height: camera.frame.3,
          output_width: width,
          output_height: height,
          radius: camera.radius,
        },
        camera.drop_shadow,
        camera.on_top,
      ))
    })
    .transpose()?;
  let prepared = crate::editor::preview_platform::annotation_gpu::prepared_arrows(
    &settings.annotations,
    (image.width, image.height),
    settings,
    Some(image),
    None,
    None,
  )?;
  let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide still canvas"),
    size: wgpu::Extent3d {
      width,
      height,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  compositor.draw_with_camera(
    &target.create_view(&Default::default()),
    &source,
    settings,
    ComposedFrame {
      cursor: inputs.cursor.map(|(cursor, _)| cursor),
      keyboard: inputs.keyboard,
      foreground_only: inputs.foreground_only,
      seconds: inputs.seconds,
    },
    camera
      .as_ref()
      .map(|(source, geometry, drop_shadow, on_top)| (source, *geometry, *drop_shadow, *on_top)),
    None,
    &prepared,
  )?;
  let mut rgba = gpu.read_texture(&target)?;
  // The canvas's colour is already faded out past its rounded corners; an
  // opaque canvas keeps full alpha there, as an encoder with no alpha sees it.
  let opaque = !inputs.transparent_background && !inputs.foreground_only;
  for pixel in rgba.as_chunks_mut::<4>().0 {
    pixel.swap(0, 2);
    if opaque {
      pixel[3] = 255;
    }
  }
  Ok(CapturedImage {
    width,
    height,
    rgba,
  })
}

/// `overlay` laid over `base`, both premultiplied RGBA of one size.
pub(crate) fn alpha_composite(
  base: &CapturedImage,
  overlay: &CapturedImage,
) -> Result<CapturedImage, String> {
  if base.width != overlay.width
    || base.height != overlay.height
    || base.rgba.len() != overlay.rgba.len()
  {
    return Err("The screenshot layers do not share a canvas size".to_owned());
  }
  let mut rgba = vec![0_u8; base.rgba.len()];
  let (below, _) = base.rgba.as_chunks::<4>();
  let (above, _) = overlay.rgba.as_chunks::<4>();
  for ((output, below), above) in rgba.as_chunks_mut::<4>().0.iter_mut().zip(below).zip(above) {
    let inverse = 1.0 - f32::from(above[3]) / 255.0;
    for channel in 0..4 {
      let value = f32::from(above[channel]) / 255.0 + f32::from(below[channel]) / 255.0 * inverse;
      output[channel] = (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    }
  }
  Ok(CapturedImage {
    width: base.width,
    height: base.height,
    rgba,
  })
}
