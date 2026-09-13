// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl CursorRaster {
  pub(super) fn sample(self, destination_x: f64, destination_y: f64, x: f64, y: f64) -> [f64; 4] {
    let dx = destination_x - x;
    let dy = destination_y - y;
    let local_x = (self.cos * dx + self.sin * dy) / self.scale + self.hotspot_x;
    let local_y = (-self.sin * dx + self.cos * dy) / self.scale + self.hotspot_y;
    let fallback_arrow = self.system_artwork.is_none() && self.artwork == fallback::Artwork::Arrow;
    // Artwork fitted by its own design frame is anchored by that frame's
    // hotspot, so it reaches outside the recorded box and is clipped by the
    // frame below instead; the fallback arrow does the same for its tip stroke.
    if !fallback_arrow
      && self.system_design.is_none()
      && (!(0.0..self.width).contains(&local_x) || !(0.0..self.height).contains(&local_y))
    {
      return [0.0; 4];
    }
    if let (Some(artwork), Some(design)) = (self.system_artwork, self.system_design) {
      // Aspect-preserving fit into the recorded box, anchored by the artwork's
      // own hotspot. Ported to the GPU by `custom_gpu_artwork` and the shader's
      // `use_design` path (gpu_compositor_macos.m `cursor_artwork_sample`).
      let artwork_scale = (self.width / design.width)
        .min(self.height / design.height)
        .max(0.01);
      let design_x = local_x / artwork_scale + design.origin_x;
      let design_y = local_y / artwork_scale + design.origin_y;
      if !(0.0..design.width).contains(&design_x) || !(0.0..design.height).contains(&design_y) {
        return [0.0; 4];
      }
      return sample_image(
        artwork,
        design_x / design.width * f64::from(artwork.width()),
        design_y / design.height * f64::from(artwork.height()),
      );
    }
    self.system_artwork.map_or_else(
      || {
        let design_size = if self.artwork == fallback::Artwork::Hand {
          (32.0, 32.0)
        } else {
          (28.0, 40.0)
        };
        let artwork_scale = (self.width / design_size.0)
          .min(self.height / design_size.1)
          .max(0.01);
        let (origin_x, origin_y) = fallback::origin(self.artwork);
        let design_x = local_x / artwork_scale + origin_x;
        let design_y = local_y / artwork_scale + origin_y;
        if fallback_arrow && (!(0.0..28.0).contains(&design_x) || !(0.0..40.0).contains(&design_y))
        {
          return [0.0; 4];
        }
        fallback::sample(self.artwork, design_x, design_y)
      },
      |artwork| {
        sample_image(
          artwork,
          local_x / self.width * f64::from(artwork.width()),
          local_y / self.height * f64::from(artwork.height()),
        )
      },
    )
  }

  pub(super) fn sample_for_draw(
    self,
    destination_x: f64,
    destination_y: f64,
    x: f64,
    y: f64,
  ) -> [f64; 4] {
    // The shader supersamples every artwork it places by a design frame
    // (`cursor_draw_sample`, gpu_compositor_macos.m), because that path clips
    // against the frame's hard edge; stretched system artwork carries its own
    // antialiased edge and is sampled once. The preview follows the same split.
    if self.system_artwork.is_some() && self.system_design.is_none() {
      return self.sample(destination_x, destination_y, x, y);
    }

    const OFFSETS: [f64; 4] = [-0.375, -0.125, 0.125, 0.375];
    let mut alpha = 0.0;
    let mut color = [0.0; 3];
    for offset_y in OFFSETS {
      for offset_x in OFFSETS {
        let source = self.sample(destination_x + offset_x, destination_y + offset_y, x, y);
        let sample_alpha = source[3] / 255.0;
        alpha += sample_alpha;
        for channel in 0..3 {
          color[channel] += source[channel] * sample_alpha;
        }
      }
    }
    if alpha <= 0.0 {
      return [0.0; 4];
    }
    for channel in &mut color {
      *channel /= alpha;
    }
    [color[0], color[1], color[2], alpha / 16.0 * 255.0]
  }
}

#[cfg(test)]
pub(super) fn sample_image(image: &RgbaImage, x: f64, y: f64) -> [f64; 4] {
  let x = x.clamp(0.0, f64::from(image.width().saturating_sub(1)));
  let y = y.clamp(0.0, f64::from(image.height().saturating_sub(1)));
  let x0 = x.floor() as u32;
  let y0 = y.floor() as u32;
  let x1 = (x0 + 1).min(image.width() - 1);
  let y1 = (y0 + 1).min(image.height() - 1);
  let fraction_x = x - f64::from(x0);
  let fraction_y = y - f64::from(y0);
  let samples = [
    (
      image.get_pixel(x0, y0).0,
      (1.0 - fraction_x) * (1.0 - fraction_y),
    ),
    (image.get_pixel(x1, y0).0, fraction_x * (1.0 - fraction_y)),
    (image.get_pixel(x0, y1).0, (1.0 - fraction_x) * fraction_y),
    (image.get_pixel(x1, y1).0, fraction_x * fraction_y),
  ];
  let mut alpha = 0.0;
  let mut color = [0.0; 3];
  for (sample, weight) in samples {
    let sample_alpha = f64::from(sample[3]) / 255.0;
    alpha += sample_alpha * weight;
    for channel in 0..3 {
      color[channel] += f64::from(sample[channel]) * sample_alpha * weight;
    }
  }
  if alpha <= 0.0 {
    return [0.0; 4];
  }
  [
    color[0] / alpha,
    color[1] / alpha,
    color[2] / alpha,
    alpha * 255.0,
  ]
}

#[cfg(test)]
impl CursorRaster {
  #[allow(clippy::too_many_arguments)]
  pub(in crate::editor::cursor_effects) fn new(
    style: CursorStyle,
    rotation_degrees: f64,
    width: f64,
    height: f64,
    hotspot_x: f64,
    hotspot_y: f64,
    scale: f64,
  ) -> Self {
    // A custom cursor's pixels are not recorded, and its box has no reason to
    // share the system arrow's aspect, so it draws the system arrow fitted
    // inside that box and anchored by the arrow's own hotspot rather than
    // stretched over it. The preview player rasterises through here and must
    // match the GPU export (`GPU_CUSTOM_ARTWORK_INDEX`, `custom_gpu_artwork`).
    let entry = platform::style_artwork(style);
    let system_artwork = entry.map(|entry| &entry.image);
    let system_design = entry
      .filter(|_| style == CursorStyle::Custom)
      .map(|entry| SystemDesign {
        height: f64::from(entry.image.height()),
        origin_x: entry.hotspot_x,
        origin_y: entry.hotspot_y,
        width: f64::from(entry.image.width()),
      });
    let vertical = system_artwork.is_none() && fallback::is_vertical(style);
    #[cfg(target_os = "windows")]
    let rotation_degrees = -rotation_degrees;
    let rotation = rotation_degrees.to_radians()
      + if vertical {
        std::f64::consts::FRAC_PI_2
      } else {
        0.0
      };
    let (sin, cos) = rotation.sin_cos();
    Self {
      artwork: fallback::artwork(style),
      cos,
      height,
      hotspot_x,
      hotspot_y,
      scale,
      sin,
      system_artwork,
      system_design,
      width,
    }
  }
}
