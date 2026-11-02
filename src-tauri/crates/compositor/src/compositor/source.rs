// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_imaging::CapturedImage;

/// How the camera's effects left a still's picture, which the canvas draws
/// it by as it draws a frame the effects pass drew on the GPU.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct PictureLook {
  /// Enhance is on: see [`SourceTexture::enhanced`].
  pub enhanced: bool,
  /// The person is cut out of their background: see
  /// [`SourceTexture::cut_out`].
  pub cut_out: bool,
}

impl Compositor {
  /// A screenshot uploaded once as the picture a canvas is drawn from.
  pub fn screenshot_source(&self, source: &CapturedImage) -> Result<SourceTexture, String> {
    check_pixels(source)?;
    let texture = self.gpu.texture_with_pixels(
      "Screenwide screenshot source",
      (source.width, source.height, 1),
      wgpu::TextureFormat::Rgba8Unorm,
      &source.rgba,
    );
    Ok(SourceTexture {
      size: (source.width, source.height),
      view: texture.create_view(&Default::default()),
      texture,
      #[cfg(target_os = "windows")]
      shared: None,
      picture: None,
      enhanced: false,
      cut_out: false,
    })
  }

  /// A still's picture, as the camera's effects left it, uploaded once and
  /// marked as `look` says. A person cut out of their background is given the
  /// mip levels their shadow is blurred from, as the effects pass gives a
  /// frame it drew, so a still casts the shadow an exported frame does.
  pub fn still_source(
    &self,
    source: &CapturedImage,
    look: PictureLook,
  ) -> Result<SourceTexture, String> {
    if !look.cut_out {
      return Ok(SourceTexture {
        enhanced: look.enhanced,
        ..self.screenshot_source(source)?
      });
    }
    check_pixels(source)?;
    let size = (source.width, source.height);
    let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Screenwide cut-out still source"),
      size: wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
      mip_level_count: screenwide_gpu::mips::levels(size),
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      // The canvas's format, which the mip levels are drawn in.
      format: FORMAT,
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let bgra: Vec<u8> = source
      .rgba
      .as_chunks::<4>()
      .0
      .iter()
      .flat_map(|&[red, green, blue, alpha]| [blue, green, red, alpha])
      .collect();
    self.gpu.queue.write_texture(
      texture.as_image_copy(),
      &bgra,
      wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(size.0 * 4),
        rows_per_image: Some(size.1),
      },
      wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
    );
    self.fill_mips(&texture);
    Ok(SourceTexture {
      size,
      view: texture.create_view(&Default::default()),
      texture,
      #[cfg(target_os = "windows")]
      shared: None,
      picture: None,
      enhanced: look.enhanced,
      cut_out: true,
    })
  }
}

fn check_pixels(source: &CapturedImage) -> Result<(), String> {
  if source.width == 0
    || source.height == 0
    || source.rgba.len() != source.width as usize * source.height as usize * 4
  {
    return Err("The screenshot preview source pixels are not valid".to_owned());
  }
  Ok(())
}
