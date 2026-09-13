// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl TextCache {
  fn invalidate(&mut self, scale: f64, light_mode: bool) {
    let identity = (scale_key(scale), light_mode);
    if self.identity == Some(identity) {
      return;
    }
    self.identity = Some(identity);
    self.labels.clear();
    self.atlas = None;
  }

  /// A whole-string texture drawn as white coverage. Chrome tints it from the
  /// portable control foreground at draw time, which is what lets one texture
  /// serve the loading and error status colours the macOS `NSTextField`
  /// carried on the view.
  pub(crate) fn label(
    &mut self,
    device: &ID3D11Device,
    text: &str,
    scale: f64,
    light_mode: bool,
    font_size: f64,
    line_height: f64,
  ) -> Option<Arc<TextTexture>> {
    self.cached(
      device,
      text,
      scale,
      light_mode,
      font_size,
      line_height,
      false,
    )
  }

  /// The ruler's tolerance notice, the one whole-string label macOS drew with
  /// Inter body text with the ink baked in,
  /// because kind 37 un-premultiplies the sample instead of tinting it.
  pub(crate) fn ink_label(
    &mut self,
    device: &ID3D11Device,
    text: &str,
    scale: f64,
    light_mode: bool,
    font_size: f64,
    line_height: f64,
  ) -> Option<Arc<TextTexture>> {
    self.cached(
      device,
      text,
      scale,
      light_mode,
      font_size,
      line_height,
      true,
    )
  }

  #[allow(clippy::too_many_arguments)]
  fn cached(
    &mut self,
    device: &ID3D11Device,
    text: &str,
    scale: f64,
    light_mode: bool,
    font_size: f64,
    line_height: f64,
    baked_ink: bool,
  ) -> Option<Arc<TextTexture>> {
    self.invalidate(scale, light_mode);
    let key = LabelKey {
      text: text.to_owned(),
      font_size: metric_key(font_size),
      line_height: metric_key(line_height),
      baked_ink,
    };
    if let Some(cached) = self.labels.get(&key) {
      return Some(Arc::clone(cached));
    }
    let ink = if !baked_ink {
      [1.0, 1.0, 1.0]
    } else if light_mode {
      LIGHT_INK
    } else {
      DARK_INK
    };
    let texture = Arc::new(build_label(
      device,
      text,
      scale,
      font_size,
      line_height,
      ink,
      false,
    )?);
    self.labels.insert(key, Arc::clone(&texture));
    Some(texture)
  }

  /// The fixed-cell Inter atlas, with the ink colour baked in the way
  /// macOS did, because its consumers draw it with the un-premultiplying
  /// glyph kind.
  pub(crate) fn hex_atlas(
    &mut self,
    device: &ID3D11Device,
    scale: f64,
    light_mode: bool,
    font_size: f64,
    line_height: f64,
  ) -> Option<Arc<TextTexture>> {
    self.invalidate(scale, light_mode);
    if let Some(cached) = self.atlas.as_ref() {
      return Some(Arc::clone(cached));
    }
    let ink = if light_mode { LIGHT_INK } else { DARK_INK };
    let texture = Arc::new(build_atlas(device, scale, font_size, line_height, ink)?);
    self.atlas = Some(Arc::clone(&texture));
    Some(texture)
  }
}
