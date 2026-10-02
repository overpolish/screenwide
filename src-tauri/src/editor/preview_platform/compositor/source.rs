// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  /// A screenshot uploaded once as the picture a canvas is drawn from.
  pub(crate) fn screenshot_source(
    &self,
    source: &crate::screenshots::CapturedImage,
  ) -> Result<SourceTexture, String> {
    if source.width == 0
      || source.height == 0
      || source.rgba.len() != source.width as usize * source.height as usize * 4
    {
      return Err("The screenshot preview source pixels are not valid".to_owned());
    }
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
    })
  }
}
