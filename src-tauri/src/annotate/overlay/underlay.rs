// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a highlight is recoloured from, as a texture the overlay's shader
//! reads: a display's underlay, or the still a highlight is baked into; and
//! the same with its mip chain for a focus's blur.

use screenwide_gpu::Gpu;
use screenwide_imaging::CapturedImage;

use super::Renderer;

/// An uploaded picture and which of Rust's revisions it is.
struct Held {
  view: wgpu::TextureView,
  revision: u64,
}

/// One display's underlay and its focus chain as the surface holds them,
/// each uploaded again only when Rust has made a new one.
#[derive(Default)]
pub(in crate::annotate) struct Underlays {
  underlay: Option<Held>,
  focus: Option<Held>,
}

impl Underlays {
  /// The underlay display `index`'s highlights read, and while a focus is on
  /// screen the same with its mip chain for the blur; `None` for either
  /// where there is none.
  pub(in crate::annotate) fn current(
    &mut self,
    renderer: &Renderer,
    index: u32,
  ) -> Result<(Option<wgpu::TextureView>, Option<wgpu::TextureView>), String> {
    let Some(current) = crate::annotate::highlight::underlay(index as usize) else {
      return Ok((None, None));
    };
    if self.underlay.as_ref().map(|held| held.revision) != Some(current.revision) {
      self.underlay = Some(Held {
        view: upload(renderer.gpu(), &current.image)?,
        revision: current.revision,
      });
    }
    if !crate::annotate::input::focus_shows() {
      self.focus = None;
    } else if self.focus.as_ref().map(|held| held.revision) != Some(current.revision) {
      self.focus = Some(Held {
        view: renderer.chains().chain(renderer.gpu(), &current.image)?,
        revision: current.revision,
      });
    }
    Ok((
      self.underlay.as_ref().map(|held| held.view.clone()),
      self.focus.as_ref().map(|held| held.view.clone()),
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
