// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One canvas composed into pixels: an exported or copied screenshot, a layer
//! of the screenshot editor, a still frame of a recording. The canvas is
//! drawn offscreen by the compositor and read back.

use std::sync::{LazyLock, Mutex};

use super::*;
use screenwide_cursor_effects::GpuCursor;
use screenwide_imaging::CapturedImage;

/// The camera bubble a still carries: its frame, the part of it shown, and
/// where that sits on the canvas, in output pixels.
pub struct StillCamera<'a> {
  pub image: &'a CapturedImage,
  /// x, y, width and height in the camera frame.
  pub crop: [u32; 4],
  /// x, y, width and height on the canvas.
  pub frame: (i32, i32, u32, u32),
  pub radius: u32,
  pub drop_shadow: bool,
  pub on_top: bool,
}

/// What one still draws besides the picture and its annotations.
pub struct StillInputs<'a> {
  pub seconds: f64,
  /// The cursor, placed on the canvas.
  pub cursor: Option<GpuCursor>,
  pub camera: Option<StillCamera<'a>>,
  pub keyboard: Option<KeyboardOverlay>,
  pub foreground_only: bool,
  /// Whether the canvas outside its rounded corners stays transparent, or is
  /// opaque black as an encoder that has no alpha would see it.
  pub transparent_background: bool,
}

/// Every still is composed through one compositor, one at a time: its
/// uniforms and lists are written for each draw before it is submitted.
static STILLS: LazyLock<Result<Mutex<Compositor>, String>> =
  LazyLock::new(|| Compositor::new(screenwide_gpu::shared()?).map(Mutex::new));

/// `image` composed on the canvas `settings` describe, as RGBA whose colour
/// is premultiplied by its alpha.
pub fn compose_still(
  image: &CapturedImage,
  settings: &ScreenshotOutputSettings,
  inputs: StillInputs<'_>,
) -> Result<CapturedImage, String> {
  let compositor = STILLS
    .as_ref()
    .map_err(Clone::clone)?
    .lock()
    .map_err(|_| "The still compositor is unavailable".to_owned())?;
  compose_still_on(&compositor, image, settings, inputs)
}

/// [`compose_still`] through `compositor`, which no other draw may be using.
pub fn compose_still_on(
  compositor: &Compositor,
  image: &CapturedImage,
  settings: &ScreenshotOutputSettings,
  inputs: StillInputs<'_>,
) -> Result<CapturedImage, String> {
  let gpu = compositor.gpu();
  let (width, height) = screenwide_output::output_dimensions(settings)?;
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
  let prepared = crate::annotation_gpu::prepared_arrows(
    &settings.annotations,
    (image.width, image.height),
    settings,
    Some(image),
    None,
    None,
  )?;
  let target = canvas_target(gpu, (width, height));
  compositor.draw_with_camera(
    &target.create_view(&Default::default()),
    &source,
    settings,
    ComposedFrame {
      cursor: inputs.cursor,
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
  // The canvas's colour is already faded out past its rounded corners; an
  // opaque canvas keeps full alpha there, as an encoder with no alpha sees it.
  let opaque = !inputs.transparent_background && !inputs.foreground_only;
  read_rgba(gpu, &target, opaque)
}

/// Screenshot items composed onto one canvas, the first with the canvas's
/// background and the rest laid over it, as the editor shows them. The
/// canvas keeps its alpha past its rounded corners.
pub fn compose_layers(
  layers: &[(&CapturedImage, ScreenshotOutputSettings)],
) -> Result<CapturedImage, String> {
  let (_, first) = layers
    .first()
    .ok_or_else(|| "The screenshot workspace is empty".to_owned())?;
  let compositor = STILLS
    .as_ref()
    .map_err(Clone::clone)?
    .lock()
    .map_err(|_| "The still compositor is unavailable".to_owned())?;
  let gpu = compositor.gpu();
  let size = screenwide_output::output_dimensions(first)?;
  let target = canvas_target(gpu, size);
  let view = target.create_view(&Default::default());
  for (index, (image, settings)) in layers.iter().enumerate() {
    if screenwide_output::output_dimensions(settings)? != size {
      return Err("The screenshot layers do not share a canvas size".to_owned());
    }
    let mut source = compositor.screenshot_source(image)?;
    source.picture = Some(std::sync::Arc::new((*image).clone()));
    // The export never carries a halo, nor a caret.
    let prepared = crate::annotation_gpu::prepared_arrows(
      &settings.annotations,
      (image.width, image.height),
      settings,
      Some(*image),
      None,
      None,
    )?;
    compositor.draw_with_camera(
      &view,
      &source,
      settings,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: index > 0,
        seconds: 0.0,
      },
      None,
      None,
      &prepared,
    )?;
  }
  read_rgba(gpu, &target, false)
}

fn canvas_target(gpu: &Gpu, (width, height): (u32, u32)) -> wgpu::Texture {
  gpu.device.create_texture(&wgpu::TextureDescriptor {
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
  })
}

/// `target`'s BGRA canvas as RGBA, fully opaque where `opaque`.
fn read_rgba(gpu: &Gpu, target: &wgpu::Texture, opaque: bool) -> Result<CapturedImage, String> {
  let mut rgba = gpu.read_texture(target)?;
  for pixel in rgba.as_chunks_mut::<4>().0 {
    pixel.swap(0, 2);
    if opaque {
      pixel[3] = 255;
    }
  }
  Ok(CapturedImage {
    width: target.width(),
    height: target.height(),
    rgba,
  })
}
