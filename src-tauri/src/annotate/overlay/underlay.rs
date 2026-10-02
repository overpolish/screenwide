// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a highlight is recoloured from, as a texture the overlay's shader
//! reads: a display's underlay, or the still a highlight is baked into; and
//! the same softened for a spotlight's blur.

use crate::gpu::Gpu;
use crate::screenshots::CapturedImage;

/// An uploaded picture and which of Rust's revisions it is.
struct Held {
  view: wgpu::TextureView,
  revision: u64,
}

/// One display's underlay and its softened copy as the surface holds them,
/// each uploaded again only when Rust has made a new one.
#[derive(Default)]
pub(in crate::annotate) struct Underlays {
  underlay: Option<Held>,
  softened: Option<Held>,
}

impl Underlays {
  /// The underlay display `index`'s highlights read, and while a spotlight
  /// blurs the same softened for its blur; `None` for either where there is
  /// none.
  pub(in crate::annotate) fn current(
    &mut self,
    gpu: &Gpu,
    index: u32,
  ) -> Result<(Option<wgpu::TextureView>, Option<wgpu::TextureView>), String> {
    let Some(current) = crate::annotate::highlight::underlay(index as usize) else {
      return Ok((None, None));
    };
    if self.underlay.as_ref().map(|held| held.revision) != Some(current.revision) {
      self.underlay = Some(Held {
        view: upload(gpu, &current.image)?,
        revision: current.revision,
      });
    }
    if !crate::annotate::input::spotlight_blurs() {
      self.softened = None;
    } else if self.softened.as_ref().map(|held| held.revision) != Some(current.revision) {
      self.softened = Some(Held {
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

/// `image` uploaded as a straight RGBA texture the shader can read.
pub(in crate::annotate) fn upload(
  gpu: &Gpu,
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
