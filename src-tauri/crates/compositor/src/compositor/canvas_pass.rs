// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::canvas_features::FeatureMask;
use super::canvas_pipelines::Target;
use super::canvas_variants::KindMask;
use super::*;

/// Which variant of the canvas shader a frame draws with: the annotation
/// kinds on it, whether every kind is held compiled, and what it uses.
#[derive(Clone, Copy)]
pub(super) struct Variant {
  pub kinds: KindMask,
  pub every: bool,
  pub features: FeatureMask,
}

/// Where a canvas pass draws: the view, its size in pixels, and whether it
/// keeps what the view held.
pub(super) type PassTarget<'a> = (&'a wgpu::TextureView, (u32, u32), wgpu::LoadOp<wgpu::Color>);

impl Compositor {
  /// One full-view draw of the canvas shader, as `pipeline` targets it.
  pub(super) fn canvas_pass(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    (view, viewport, load): PassTarget<'_>,
    pipeline: Target,
    variant: Variant,
    bindings: &wgpu::BindGroup,
  ) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Screenwide preview canvas"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load,
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });
    pass.set_viewport(0.0, 0.0, viewport.0 as f32, viewport.1 as f32, 0.0, 1.0);
    pass.set_pipeline(
      &self
        .canvas
        .get(pipeline, variant.kinds, variant.every, variant.features),
    );
    pass.set_bind_group(0, bindings, &[]);
    pass.draw(0..3, 0..1);
  }
}
