// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What one focused frame's blur asks: the boxes in the shape the gather
//! reads them, where the topmost sits among the annotations, and the
//! constants each canvas pass of the frame is drawn with.

use super::super::constants::Constants;
use crate::annotation_gpu::{PreparedArrows, PreviewArrow};
use screenwide_annotations::AnnotationKind;

/// How wide the blur reaches at the middle strength, as a share of the
/// canvas's shorter side: enough to read as a lens without smearing what is
/// around the box out of recognition.
const WIDEST_SHARE: f32 = 0.013;
/// How far past a box the blur takes to arrive, as a share of the canvas's
/// shorter side.
const REACH_SHARE: f32 = 0.45;
/// How wide the depth term reaches as far from the box's depth as the turn
/// carries the screen, as a share of the canvas's shorter side.
const DEPTH_SHARE: f32 = 0.05;
/// How far past the screen's edge the background takes to reach the whole
/// blur, from the blur the screen has at its edge, as a share of the
/// canvas's shorter side: short enough that the background still reads as
/// lying well beyond, long enough that the edge leaves no seam.
const EDGE_SHARE: f32 = 0.1;

/// The canvas pass's `focus` modes.
const UNDER: f32 = 1.0;
const OVER: f32 = 2.0;

/// The twin of `FocusRegion` in `focus_blur.wgsl`, a 32-byte stride.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct GpuRegion {
  bounds: [f32; 4],
  radius: f32,
  strength: f32,
  presence: f32,
  reference: f32,
}

/// The twin of `FocusConstants` in `focus_blur.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct FocusConstants {
  output_source: [f32; 4],
  image_rect: [f32; 4],
  crop_rect: [f32; 4],
  placement: [f32; 4],
  inverse: [[f32; 4]; 3],
  shape: [f32; 4],
  edge: [f32; 4],
}

/// One focused frame's blur.
pub(in super::super) struct FocusPlan {
  pub(super) constants: FocusConstants,
  pub(super) regions: Vec<GpuRegion>,
  cut: u32,
  /// Whether the focus lies in the run above the camera.
  above_camera: bool,
}

impl FocusPlan {
  /// The blur `prepared` asks of a frame drawn with `values`, or `None`
  /// where no focus is showing.
  pub(in super::super) fn of(values: &Constants, prepared: &PreparedArrows) -> Option<Self> {
    if prepared.focus.is_empty() {
      return None;
    }
    let [width, height, source_width, source_height] = values.output_source;
    let shorter = width.min(height).max(1.0);
    let [ix, iy, iw, ih] = values.image_rect;
    let row = |index: usize| {
      let [a, b, c, _] = values.scene_turn[index];
      [a, b, c]
    };
    let dot = |row: [f32; 3], [x, y]: [f32; 2]| row[0] * x + row[1] * y + row[2];
    let regions = prepared
      .focus
      .iter()
      .map(|region| {
        let [x0, y0, x1, y1] = region.bounds;
        // The box's middle on the upright canvas, carried by the turn to
        // where it is drawn, then read back as the inverse turn's weight
        // there: the depth every other point is measured from.
        let upright = [
          ix + (x0 + x1) * 0.5 * iw / source_width.max(1.0),
          iy + (y0 + y1) * 0.5 * ih / source_height.max(1.0),
        ];
        let weight = dot(row(5), upright);
        let reference = if weight > 1e-6 {
          let drawn = [dot(row(3), upright) / weight, dot(row(4), upright) / weight];
          dot(row(2), drawn)
        } else {
          1.0
        };
        GpuRegion {
          bounds: region.bounds,
          radius: region.radius,
          strength: region.strength,
          presence: region.presence,
          reference,
        }
      })
      .collect::<Vec<_>>();
    Some(Self {
      constants: FocusConstants {
        output_source: values.output_source,
        image_rect: values.image_rect,
        crop_rect: values.crop_rect,
        placement: values.placement,
        inverse: [
          values.scene_turn[0],
          values.scene_turn[1],
          values.scene_turn[2],
        ],
        shape: [
          REACH_SHARE * shorter,
          WIDEST_SHARE * shorter,
          DEPTH_SHARE * shorter,
          regions.len() as f32,
        ],
        edge: [
          EDGE_SHARE * shorter,
          values.effects[1],
          // A layer laid over others keeps its own alpha.
          f32::from(u8::from(values.cursor_options[3] == 0)),
          0.0,
        ],
      },
      regions,
      cut: prepared.focus_cut,
      above_camera: prepared.focus_above_camera,
    })
  }

  /// `values` for the canvas pass that draws what lies under the focus (`0`)
  /// or over it (`1`).
  pub(in super::super) fn pass(&self, mut values: Constants, part: usize) -> Constants {
    values.focus = [
      if part == 0 { UNDER } else { OVER },
      self.cut as f32,
      f32::from(u8::from(self.above_camera)),
      0.0,
    ];
    values
  }

  /// Whether anything lies over the focus for the pass over it to draw: an
  /// annotation after it, its own halo, the shortcut strip, the crop
  /// magnifier, or a pane drawn after it - the camera in front of a focus on
  /// the screen, or the screen in front of one on the camera. Most focused
  /// frames have none, and that pass runs the whole canvas shader again.
  pub(in super::super) fn draws_over(
    &self,
    values: &Constants,
    arrows: &[PreviewArrow],
    keyboard: bool,
  ) -> bool {
    let focus = AnnotationKind::Focus.raw();
    let annotation_over = arrows
      .iter()
      .skip(self.cut as usize)
      .any(|arrow| arrow.kind != focus || arrow.hover > 0.0);
    let camera = values.camera_effects[1] != 0.0 && values.scene_opacity[1] > 0.0;
    let camera_front = values.camera_effects[3];
    let pane_over = camera
      && if self.above_camera {
        camera_front < 1.0
      } else {
        camera_front > 0.0
      };
    annotation_over || keyboard || values.magnifier[2] > 0.0 || pane_over
  }
}
