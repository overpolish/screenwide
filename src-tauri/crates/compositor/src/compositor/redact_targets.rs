// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The textures the redaction passes draw into and read back.

use super::*;

/// A texture a pass draws into and the next one reads.
pub(super) struct Target {
  pub(super) texture: wgpu::Texture,
  pub(super) view: wgpu::TextureView,
}

pub(super) fn target(gpu: &Gpu, size: (u32, u32), format: wgpu::TextureFormat) -> Target {
  let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide redaction target"),
    size: wgpu::Extent3d {
      width: size.0.max(1),
      height: size.1.max(1),
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format,
    usage: wgpu::TextureUsages::TEXTURE_BINDING
      | wgpu::TextureUsages::RENDER_ATTACHMENT
      | wgpu::TextureUsages::COPY_SRC
      | wgpu::TextureUsages::COPY_DST,
    view_formats: &[],
  });
  let view = texture.create_view(&Default::default());
  Target { texture, view }
}

/// A target a pass draws into before the paint pass reads it, grown to fit
/// the largest it has been asked for, and never shrunk.
pub(super) struct Scratch {
  format: wgpu::TextureFormat,
  slot: std::sync::Mutex<Option<((u32, u32), wgpu::TextureView)>>,
}

impl Scratch {
  pub(super) fn new(format: wgpu::TextureFormat) -> Self {
    Self {
      format,
      slot: std::sync::Mutex::new(None),
    }
  }

  pub(super) fn format(&self) -> wgpu::TextureFormat {
    self.format
  }

  /// The target, first grown to at least `size`.
  pub(super) fn view(&self, gpu: &Gpu, size: (u32, u32)) -> Result<wgpu::TextureView, String> {
    let mut slot = self
      .slot
      .lock()
      .map_err(|_| "A redaction scratch target is poisoned".to_owned())?;
    if slot
      .as_ref()
      .is_none_or(|(held, _)| held.0 < size.0 || held.1 < size.1)
    {
      let grown = slot
        .as_ref()
        .map_or(size, |(held, _)| (held.0.max(size.0), held.1.max(size.1)));
      *slot = Some((grown, target(gpu, grown, self.format).view));
    }
    let (_, view) = slot.as_ref().expect("the scratch target was just made");
    Ok(view.clone())
  }
}
