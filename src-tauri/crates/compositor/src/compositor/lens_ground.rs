// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What each lens shows, composed ahead of the canvas: a pass of the canvas
//! shader draws every lens's ground into one atlas, and the canvas pass reads
//! each look of each lens from there (`annotation_lens.wgsl`).
//!
//! A lens settling onto the picture is drawn whole at every exposure sample,
//! and composing what it shows - the background, the screen with its crop,
//! corners and shadow, and every annotation below it - wherever each sample
//! looked cost the 1080p preview tens of milliseconds a frame. Composed once
//! a texel here, each look is one read.
//!
//! A ground covers the points every sample of its lens shows, one texel for
//! each pixel the lens draws as the frame places it, and its texels lie on
//! the points the lens shows at its drawn pixels' centres, so a still lens
//! reads each texel whole and shows what it showed before.

use super::*;
use crate::annotation_gpu::{PreparedArrows, PreviewArrow};
use screenwide_annotations::AnnotationKind;

/// The longest side one lens's ground takes, in texels; a lens drawn larger
/// is read from a coarser ground.
const LONGEST_GROUND: f32 = 4096.0;

/// Where the grounds lie: the atlas's size in texels, and the part of the
/// target they read the layers under them from, x, y, width and height.
pub(super) struct GroundPlan {
  size: (u32, u32),
  pub(super) under: [u32; 4],
}

impl GroundPlan {
  /// Lays a ground for each lens in `arrows`, which `prepared` holds with
  /// their exposure samples and which draw `kinds`, into one atlas, and
  /// writes into each lens's record where its ground lies; `None` where
  /// there is no lens. `focused` says whether a focus cuts the frame, and
  /// `target` is the size of what the canvas is drawn into.
  pub(super) fn of(
    kinds: super::canvas_variants::KindMask,
    arrows: &mut [PreviewArrow],
    prepared: &PreparedArrows,
    values: &Constants,
    focused: bool,
    target: (u32, u32),
  ) -> Option<Self> {
    if kinds & super::canvas_variants::kind_bit(AnnotationKind::Magnify) == 0 {
      return None;
    }
    let samples = &prepared.samples;
    let focus = focused.then_some((prepared.focus_cut, prepared.focus_above_camera));
    let magnify = AnnotationKind::Magnify.raw();
    let below_camera = values.annotation_options[0];
    let [x, y, per_x, per_y] = values.placement;
    let corner = [x, y];
    let per = [per_x.max(1e-6), per_y.max(1e-6)];
    let feather = 0.5 * values.motion[2].max(1e-4);
    let (mut shelf, mut row) = ([0.0_f32; 2], 0.0_f32);
    let mut reach: Option<[f32; 4]> = None;
    for (index, arrow) in arrows.iter_mut().enumerate() {
      if arrow.kind != magnify {
        continue;
      }
      let first = arrow.sample_first as usize;
      let taps = samples
        .get(first..first + arrow.sample_count as usize)
        .unwrap_or_default();
      // Every point a look of a sample can land on lies inside its lens.
      let seen = std::iter::once(&arrow.geometry)
        .chain(taps.iter().map(|sample| &sample.geometry))
        .filter(|shape| shape.bx > 0.0 && shape.by > 0.0)
        .map(|shape| {
          let zoom = shape.high.max(1e-3);
          let half = [
            (shape.bx + 2.0 * feather) / zoom,
            (shape.by + 2.0 * feather) / zoom,
          ];
          [
            shape.ax - half[0],
            shape.ay - half[1],
            shape.ax + half[0],
            shape.ay + half[1],
          ]
        })
        .reduce(spanning);
      let Some(seen) = seen else {
        continue;
      };
      reach = Some(reach.map_or(seen, |reach| spanning(reach, seen)));
      // The texel grid lies on the points this lens shows at the target's
      // pixel centres, `step` apart.
      let zoom = arrow.geometry.high.max(1e-3);
      let centre = [arrow.geometry.ax, arrow.geometry.ay];
      let mut step = [per[0] / zoom, per[1] / zoom];
      let mut origin = [0.0; 2];
      let mut size = [0.0; 2];
      for axis in 0..2 {
        let pixel =
          ((centre[axis] + (seen[axis] - centre[axis]) * zoom) / per[axis] + corner[axis] - 0.5)
            .floor();
        origin[axis] =
          centre[axis] + ((pixel + 0.5 - corner[axis]) * per[axis] - centre[axis]) / zoom;
        size[axis] = ((seen[axis + 2] - origin[axis]) / step[axis]).floor() + 2.0;
      }
      let longest = size[0].max(size[1]);
      if longest > LONGEST_GROUND {
        let coarser = longest / LONGEST_GROUND;
        for axis in 0..2 {
          step[axis] *= coarser;
          origin[axis] = seen[axis];
          size[axis] =
            (((seen[axis + 2] - seen[axis]) / step[axis]).ceil() + 1.0).min(LONGEST_GROUND);
        }
      }
      if shelf[0] + size[0] > LONGEST_GROUND * 2.0 {
        shelf = [0.0, shelf[1] + row];
        row = 0.0;
      }
      // Past the atlas's last row a lens shows nothing, which only far
      // more lenses than a picture can hold come to.
      if shelf[1] + size[1] > LONGEST_GROUND * 2.0 {
        continue;
      }
      let camera_run = index as u32 >= below_camera;
      // A lens drawn by the pass over a focus in its own run is drawn
      // without the cursor, which lies under the focus.
      let over_focus =
        focus.is_some_and(|(cut, above_camera)| above_camera == camera_run && index as u32 >= cut);
      let cursor = !camera_run && !over_focus;
      arrow.geometry.turn[..10].copy_from_slice(&[
        origin[0],
        origin[1],
        step[0],
        step[1],
        shelf[0],
        shelf[1],
        size[0],
        size[1],
        f32::from(u8::from(camera_run)),
        f32::from(u8::from(cursor)),
      ]);
      shelf[0] += size[0];
      row = row.max(size[1]);
    }
    let reach = reach?;
    let size = (
      (shelf[0].max(1.0) as u32).max(1),
      ((shelf[1] + row).max(1.0) as u32).max(1),
    );
    // The layers under the grounds, in target pixels and a little past, or
    // the whole target where the screen turns and its points land anywhere.
    let turned = values.scene_turn[0][3] != 0.0;
    let under = if turned {
      [0, 0, target.0, target.1]
    } else {
      let low = |axis: usize, side: u32| {
        ((reach[axis] / per[axis] + corner[axis]).floor() - 2.0).clamp(0.0, side as f32) as u32
      };
      let high = |axis: usize, side: u32| {
        ((reach[axis + 2] / per[axis] + corner[axis]).ceil() + 2.0).clamp(0.0, side as f32) as u32
      };
      let (left, top) = (low(0, target.0), low(1, target.1));
      [left, top, high(0, target.0) - left, high(1, target.1) - top]
    };
    Some(Self { size, under })
  }
}

/// The box spanning both `a` and `b`, each low corner then high.
fn spanning(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
  [
    a[0].min(b[0]),
    a[1].min(b[1]),
    a[2].max(b[2]),
    a[3].max(b[3]),
  ]
}

/// The atlas the grounds are drawn into, kept between frames and made again
/// only when a frame's grounds outgrow it or need far less, and the
/// constants each pass drawing them reads.
pub(super) struct LensGround {
  atlas: std::sync::Mutex<Option<wgpu::Texture>>,
  /// For the pass the marks' blur layer reads, before that layer is drawn,
  /// then for the canvas's.
  pub(super) constants: [wgpu::Buffer; 2],
}

impl LensGround {
  pub(super) fn new(device: &wgpu::Device) -> Self {
    let uniform = |label| {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: std::mem::size_of::<Constants>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };
    Self {
      atlas: std::sync::Mutex::new(None),
      constants: [
        uniform("Screenwide lens grounds under the mark blur"),
        uniform("Screenwide lens grounds"),
      ],
    }
  }

  /// Writes the constants of the pass in `slot` drawing the grounds of a
  /// canvas drawn with `values`: the first reads no blurred layer, which is
  /// drawn after it.
  pub(super) fn write(&self, gpu: &Gpu, slot: usize, mut values: Constants) {
    values.overflow[1] = 1.0;
    if slot == 0 {
      values.annotation_blur[1] = 0.0;
    }
    gpu
      .queue
      .write_buffer(&self.constants[slot], 0, bytemuck::bytes_of(&values));
  }

  /// A view of the atlas, big enough for `plan`.
  pub(super) fn atlas(&self, gpu: &Gpu, plan: &GroundPlan) -> Result<wgpu::TextureView, String> {
    let mut atlas = self
      .atlas
      .lock()
      .map_err(|_| "The lenses' grounds are poisoned".to_owned())?;
    let (width, height) = plan.size;
    let fits = atlas.as_ref().is_some_and(|held| {
      held.width() >= width
        && held.height() >= height
        && u64::from(held.width()) * u64::from(held.height())
          <= 4 * u64::from(width) * u64::from(height)
    });
    if !fits {
      let round = |side: u32| side.div_ceil(256) * 256;
      *atlas = Some(gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenwide lens grounds"),
        size: wgpu::Extent3d {
          width: round(width),
          height: round(height),
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: super::mark_blur::LAYER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      }));
    }
    atlas
      .as_ref()
      .map(|held| held.create_view(&Default::default()))
      .ok_or_else(|| "The lenses' grounds are missing".to_owned())
  }

  /// Records the pass drawing `plan`'s grounds into `atlas` with `pipeline`
  /// and `bindings`.
  pub(super) fn draw(
    encoder: &mut wgpu::CommandEncoder,
    atlas: &wgpu::TextureView,
    plan: &GroundPlan,
    pipeline: &wgpu::RenderPipeline,
    bindings: &wgpu::BindGroup,
  ) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Screenwide lens grounds"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: atlas,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });
    pass.set_viewport(0.0, 0.0, plan.size.0 as f32, plan.size.1 as f32, 0.0, 1.0);
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bindings, &[]);
    pass.draw(0..3, 0..1);
  }
}
