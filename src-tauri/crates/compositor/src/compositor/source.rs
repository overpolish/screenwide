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

  /// The part `[x0, y0, x1, y1)` of a screenshot too large for the device,
  /// uploaded as the picture a canvas is drawn from, each texel the average
  /// of `scale` by `scale` of its pixels. Its `size` is the whole
  /// screenshot's; draw it with the layer's `source_window` at the window's
  /// corner and `scale`.
  pub fn screenshot_window(
    &self,
    source: &CapturedImage,
    [x0, y0, x1, y1]: [u32; 4],
    scale: u32,
  ) -> Result<SourceTexture, String> {
    check_pixels(source)?;
    if x0 >= x1 || y0 >= y1 || x1 > source.width || y1 > source.height || scale == 0 {
      return Err("The screenshot window lies outside the screenshot".to_owned());
    }
    let extent = wgpu::Extent3d {
      width: (x1 - x0).div_ceil(scale),
      height: (y1 - y0).div_ceil(scale),
      depth_or_array_layers: 1,
    };
    let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Screenwide screenshot window"),
      size: extent,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      // Copyable so a redaction pass can start from its pixels.
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::COPY_DST
        | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let layout = |offset, bytes_per_row| wgpu::TexelCopyBufferLayout {
      offset,
      bytes_per_row: Some(bytes_per_row),
      rows_per_image: Some(extent.height),
    };
    if scale == 1 {
      // Read straight out of the whole screenshot's rows.
      let offset = (u64::from(y0) * u64::from(source.width) + u64::from(x0)) * 4;
      let layout = layout(offset, source.width * 4);
      self
        .gpu
        .queue
        .write_texture(texture.as_image_copy(), &source.rgba, layout, extent);
    } else {
      let averaged = averaged(source, [x0, y0, x1, y1], scale, extent)?;
      let layout = layout(0, extent.width * 4);
      self
        .gpu
        .queue
        .write_texture(texture.as_image_copy(), &averaged, layout, extent);
    }
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

/// The `extent` texels of `source`'s part `[x0, y0, x1, y1)`, each the mean
/// of the `scale` by `scale` pixels it spans there.
fn averaged(
  source: &CapturedImage,
  [x0, y0, x1, y1]: [u32; 4],
  scale: u32,
  extent: wgpu::Extent3d,
) -> Result<Vec<u8>, String> {
  let mut texels = Vec::new();
  texels
    .try_reserve_exact(extent.width as usize * extent.height as usize * 4)
    .map_err(|_| "There is not enough memory to show the screenshot".to_owned())?;
  let mut sums = vec![[0_u32; 4]; extent.width as usize];
  for row in 0..extent.height {
    sums.fill([0; 4]);
    let top = y0 + row * scale;
    let bottom = (top + scale).min(y1);
    for y in top..bottom {
      let start = (y as usize * source.width as usize + x0 as usize) * 4;
      let line = &source.rgba[start..start + (x1 - x0) as usize * 4];
      for (x, pixel) in line.as_chunks::<4>().0.iter().enumerate() {
        let sum = &mut sums[x / scale as usize];
        for channel in 0..4 {
          sum[channel] += u32::from(pixel[channel]);
        }
      }
    }
    for (column, sum) in sums.iter().enumerate() {
      let left = x0 + column as u32 * scale;
      let count = (bottom - top) * ((left + scale).min(x1) - left);
      texels.extend(sum.map(|total| ((total + count / 2) / count.max(1)) as u8));
    }
  }
  Ok(texels)
}
