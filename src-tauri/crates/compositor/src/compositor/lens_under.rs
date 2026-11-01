// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a lens on a layer laid over others shows of them: a copy of the
//! target as it stood before the layer was drawn. The canvas pass reads the
//! layers under a lens from it, so a lens shows the finished picture under
//! it and not only its own layer. A layer drawn first, with the background,
//! needs none: the canvas pass draws its background itself.

use super::*;

/// The copy, kept between frames and made again only when the target's
/// size or format changes. Only the part the lenses read is copied; the
/// rest holds whatever an earlier frame left, which nothing reads.
#[derive(Default)]
pub(super) struct LensUnder {
  copy: std::sync::Mutex<Option<wgpu::Texture>>,
}

impl LensUnder {
  /// Records a copy of `target` as it stands where the lenses read it, the
  /// part `[x, y, width, height]`, and answers a view of it, or `None` where
  /// the target cannot be read back, which leaves a lens on it showing
  /// nothing of the layers under it.
  pub(super) fn record(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::Texture,
    [x, y, width, height]: [u32; 4],
  ) -> Result<Option<wgpu::TextureView>, String> {
    if !target.usage().contains(wgpu::TextureUsages::COPY_SRC) {
      return Ok(None);
    }
    let mut copy = self
      .copy
      .lock()
      .map_err(|_| "The copy under the lenses is poisoned".to_owned())?;
    let fits = copy
      .as_ref()
      .is_some_and(|held| held.size() == target.size() && held.format() == target.format());
    if !fits {
      *copy = Some(gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenwide layers under a lens"),
        size: target.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target.format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
      }));
    }
    let Some(held) = copy.as_ref() else {
      return Ok(None);
    };
    fn part(texture: &wgpu::Texture, x: u32, y: u32) -> wgpu::TexelCopyTextureInfo<'_> {
      wgpu::TexelCopyTextureInfo {
        origin: wgpu::Origin3d { x, y, z: 0 },
        ..texture.as_image_copy()
      }
    }
    if width == 0 || height == 0 {
      return Ok(Some(held.create_view(&Default::default())));
    }
    encoder.copy_texture_to_texture(
      part(target, x, y),
      part(held, x, y),
      wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
    );
    Ok(Some(held.create_view(&Default::default())))
  }
}
