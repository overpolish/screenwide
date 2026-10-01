// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a highlight is recoloured from, as a texture the overlay's shader
//! reads: a display's underlay, or the still a highlight is baked into; and
//! the same softened for a spotlight's blur.

use super::*;

use crate::screenshots::CapturedImage;

/// A display's underlay as the surface holds it: the view the shader reads,
/// and which of Rust's revisions it is.
pub(super) struct HeldUnderlay {
  pub(super) view: wgpu::TextureView,
  pub(super) revision: u64,
}

/// `image` uploaded as a straight RGBA texture the shader can read.
pub(super) fn upload(
  gpu: &crate::gpu::Gpu,
  image: &CapturedImage,
) -> Result<wgpu::TextureView, String> {
  if image.width == 0 || image.height == 0 {
    return Err("A highlight's underlay has no pixels".to_owned());
  }
  Ok(
    gpu
      .texture_with_pixels(
        "Screenwide highlight underlay",
        (image.width, image.height, 1),
        wgpu::TextureFormat::Rgba8Unorm,
        &image.rgba,
      )
      .create_view(&Default::default()),
  )
}

impl Surface {
  /// The underlay this display's highlights read, uploaded again when Rust
  /// has made a new one, and while a spotlight blurs, the same softened for
  /// its blur; `None` for either where there is none.
  pub(super) fn underlay(
    &mut self,
    gpu: &crate::gpu::Gpu,
  ) -> Result<(Option<wgpu::TextureView>, Option<wgpu::TextureView>), String> {
    let Some(current) = super::super::super::highlight::underlay(self.display as usize) else {
      return Ok((None, None));
    };
    if self.underlay.as_ref().map(|held| held.revision) != Some(current.revision) {
      self.underlay = Some(HeldUnderlay {
        view: upload(gpu, &current.image)?,
        revision: current.revision,
      });
    }
    if !super::super::super::input::spotlight_blurs() {
      self.softened = None;
    } else if self.softened.as_ref().map(|held| held.revision) != Some(current.revision) {
      self.softened = Some(HeldUnderlay {
        view: upload(gpu, current.softened())?,
        revision: current.revision,
      });
    }
    Ok((
      self.underlay.as_ref().map(|held| held.view.clone()),
      self.softened.as_ref().map(|held| held.view.clone()),
    ))
  }
}
