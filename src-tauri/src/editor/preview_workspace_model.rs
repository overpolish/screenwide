// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Converts screenshot and recording settings into the one native workspace
//! model consumed by both GPU backends.

use super::preview_platform::workspace_editor::{
  FrameId, LayerGeometry, LayerId, NormalizedRect, WorkspaceFrame, WorkspaceLayer, WorkspaceScene,
  WorldRect,
};
use super::{CameraOverlaySettings, RecordingOutputSettings, ScreenshotWorkspaceOutputSettings};
use crate::screenshots::ScreenshotOutputSettings;

#[derive(Clone, Copy)]
pub(crate) struct WorkspacePane {
  pub id: u32,
  pub rect: WorldRect,
}

/// The canvas a placement is measured against, never smaller than one pixel.
pub(crate) fn output_canvas(output: &ScreenshotOutputSettings) -> (f64, f64) {
  (
    f64::from(output.width.max(1)),
    f64::from(output.height.max(1)),
  )
}

fn layer(
  id: u32,
  frame_id: u32,
  output: &ScreenshotOutputSettings,
  z_index: i32,
) -> WorkspaceLayer {
  let (width, height) = output_canvas(output);
  WorkspaceLayer {
    id: LayerId(id),
    frame_id: FrameId(frame_id),
    rect: NormalizedRect {
      x: output.crop_x / width,
      y: output.crop_y / height,
      width: output.crop_width / width,
      height: output.crop_height / height,
    },
    radius_percent: output.radius_percent,
    z_index,
  }
}

/// A layer's stored pixel placement, normalized to the canvas the scene draws.
pub(crate) fn output_geometry(output: &ScreenshotOutputSettings) -> LayerGeometry {
  let (width, height) = output_canvas(output);
  LayerGeometry {
    crop: NormalizedRect {
      x: output.crop_x / width,
      y: output.crop_y / height,
      width: output.crop_width / width,
      height: output.crop_height / height,
    },
    image_width: output.image_width / width,
    image_x: output.image_x / width,
    image_y: output.image_y / height,
    radius_percent: output.radius_percent,
  }
}

/// Write a scene geometry back as the pixels the settings store.
pub(crate) fn apply_output_geometry(
  output: &mut ScreenshotOutputSettings,
  geometry: LayerGeometry,
) {
  let (width, height) = output_canvas(output);
  output.crop_x = geometry.crop.x * width;
  output.crop_y = geometry.crop.y * height;
  output.crop_width = geometry.crop.width * width;
  output.crop_height = geometry.crop.height * height;
  output.image_width = geometry.image_width * width;
  output.image_x = geometry.image_x * width;
  output.image_y = geometry.image_y * height;
  output.radius_percent = geometry.radius_percent;
}

pub(crate) fn camera_geometry(camera: CameraOverlaySettings, canvas: (f64, f64)) -> LayerGeometry {
  LayerGeometry {
    crop: NormalizedRect {
      x: camera.frame_x / canvas.0,
      y: camera.frame_y / canvas.1,
      width: camera.frame_width / canvas.0,
      height: camera.frame_height / canvas.1,
    },
    image_width: camera.camera_width / canvas.0,
    image_x: camera.camera_x / canvas.0,
    image_y: camera.camera_y / canvas.1,
    radius_percent: camera.radius_percent,
  }
}

pub(crate) fn apply_camera_geometry(
  camera: &mut CameraOverlaySettings,
  geometry: LayerGeometry,
  canvas: (f64, f64),
) {
  camera.frame_x = geometry.crop.x * canvas.0;
  camera.frame_y = geometry.crop.y * canvas.1;
  camera.frame_width = geometry.crop.width * canvas.0;
  camera.frame_height = geometry.crop.height * canvas.1;
  camera.camera_width = geometry.image_width * canvas.0;
  camera.camera_x = geometry.image_x * canvas.0;
  camera.camera_y = geometry.image_y * canvas.1;
  camera.radius_percent = geometry.radius_percent;
}

pub(crate) fn screenshot_scene(
  viewport: WorldRect,
  output: &ScreenshotWorkspaceOutputSettings,
  revision: u64,
) -> Result<WorkspaceScene, String> {
  let layers = output
    .items
    .iter()
    .enumerate()
    .map(|(index, item)| layer(index as u32, 0, &item.output, index as i32))
    .collect();
  let mut scene = WorkspaceScene::screenshot(
    viewport,
    WorldRect {
      x: 0.0,
      y: 0.0,
      width: f64::from(output.canvas.width),
      height: f64::from(output.canvas.height),
    },
    layers,
  )?;
  scene.frames[0].radius_percent = output.canvas.background_radius_percent;
  scene.revision = revision;
  Ok(scene)
}

pub(crate) fn recording_scene(
  viewport: WorldRect,
  panes: &[WorkspacePane],
  bake_camera: bool,
  camera: CameraOverlaySettings,
  output: &RecordingOutputSettings,
  revision: u64,
) -> Result<WorkspaceScene, String> {
  let mut scene = if bake_camera {
    let frame_id = panes
      .iter()
      .find(|pane| pane.id == 0)
      .or_else(|| panes.first())
      .ok_or_else(|| "recording workspace must contain a frame".to_owned())?
      .id;
    let primary_canvas = output_canvas(&output.primary);
    let layers = vec![
      layer(0, frame_id, &output.primary, 0),
      WorkspaceLayer {
        id: LayerId(1),
        frame_id: FrameId(frame_id),
        rect: NormalizedRect {
          x: camera.frame_x / primary_canvas.0,
          y: camera.frame_y / primary_canvas.1,
          width: camera.frame_width / primary_canvas.0,
          height: camera.frame_height / primary_canvas.1,
        },
        radius_percent: camera.radius_percent,
        z_index: i32::from(output.camera_on_top),
      },
    ];
    WorkspaceScene::baked_video(
      viewport,
      WorldRect {
        x: 0.0,
        y: 0.0,
        width: f64::from(output.primary.width),
        height: f64::from(output.primary.height),
      },
      layers,
    )?
  } else {
    let left = panes
      .iter()
      .map(|pane| pane.rect.x)
      .fold(f64::INFINITY, f64::min);
    let top = panes
      .iter()
      .map(|pane| pane.rect.y)
      .fold(f64::INFINITY, f64::min);
    let fit = panes
      .iter()
      .find_map(|pane| {
        let width = if pane.id == 0 {
          output.primary.width
        } else {
          output.camera.width
        };
        (width > 0 && pane.rect.width > 0.0).then_some(pane.rect.width / f64::from(width))
      })
      .unwrap_or(1.0)
      .max(f64::EPSILON);
    let frames = panes
      .iter()
      .map(|pane| {
        let output = if pane.id == 0 {
          &output.primary
        } else {
          &output.camera
        };
        WorkspaceFrame {
          id: FrameId(pane.id),
          rect: WorldRect {
            x: (pane.rect.x - left) / fit,
            y: (pane.rect.y - top) / fit,
            width: f64::from(output.width),
            height: f64::from(output.height),
          },
          radius_percent: 0.0,
        }
      })
      .collect();
    let layers = panes
      .iter()
      .map(|pane| {
        let output = if pane.id == 0 {
          &output.primary
        } else {
          &output.camera
        };
        layer(pane.id, pane.id, output, 0)
      })
      .collect();
    WorkspaceScene::split_video(viewport, frames, layers)?
  };
  scene.revision = revision;
  Ok(scene)
}

pub(crate) fn resize_screenshot_frame(
  scene: &WorkspaceScene,
  output: &ScreenshotWorkspaceOutputSettings,
  edges: u32,
  normalized_delta: (f64, f64),
) -> Result<(WorkspaceScene, ScreenshotWorkspaceOutputSettings), String> {
  let frame = scene.frame(FrameId(0)).ok_or("screenshot frame missing")?;
  let resized = scene.resized_frame(
    FrameId(0),
    edges,
    (
      normalized_delta.0 * frame.rect.width,
      normalized_delta.1 * frame.rect.height,
    ),
  )?;
  // Only the canvas changes size, and nothing in it moves on screen. When a
  // near edge or a centred resize moves the canvas's corner, placement, which
  // is measured from that corner, is renumbered by the amount it moved.
  let shift = origin_shift(frame.rect, resized.new_rect);
  let mut next = output.clone();
  next.canvas.width = resized.output_size.0;
  next.canvas.height = resized.output_size.1;
  for item in &mut next.items {
    item.output.width = resized.output_size.0;
    item.output.height = resized.output_size.1;
    shift_output(&mut item.output, shift);
  }
  Ok((resized.scene, next))
}

/// How far a frame's top left corner moved, in output pixels.
fn origin_shift(old: WorldRect, new: WorldRect) -> (f64, f64) {
  (new.x - old.x, new.y - old.y)
}

/// Renumbers a layer's placement so it keeps its place on screen after the
/// canvas's corner moved by `shift`.
fn shift_output(output: &mut ScreenshotOutputSettings, shift: (f64, f64)) {
  output.crop_x -= shift.0;
  output.crop_y -= shift.1;
  output.image_x -= shift.0;
  output.image_y -= shift.1;
}

pub(crate) fn resize_recording_frame(
  scene: &WorkspaceScene,
  output: &RecordingOutputSettings,
  camera: CameraOverlaySettings,
  frame_id: u32,
  edges: u32,
  normalized_delta: (f64, f64),
) -> Result<
  (
    WorkspaceScene,
    RecordingOutputSettings,
    CameraOverlaySettings,
  ),
  String,
> {
  let id = FrameId(frame_id);
  let frame = scene.frame(id).ok_or("recording frame missing")?;
  let resized = scene.resized_frame(
    id,
    edges,
    (
      normalized_delta.0 * frame.rect.width,
      normalized_delta.1 * frame.rect.height,
    ),
  )?;
  // Only the canvas changes size, and nothing in it moves on screen: the
  // track's placement and a baked camera overlay are renumbered by however
  // far the canvas's corner moved, which a near edge or a centred resize does.
  let shift = origin_shift(frame.rect, resized.new_rect);
  let mut next_output = output.clone();
  let mut camera = camera;
  if frame_id == 0 {
    camera.frame_x -= shift.0;
    camera.frame_y -= shift.1;
    camera.camera_x -= shift.0;
    camera.camera_y -= shift.1;
  }
  let selected = if frame_id == 0 {
    &mut next_output.primary
  } else {
    &mut next_output.camera
  };
  selected.width = resized.output_size.0;
  selected.height = resized.output_size.1;
  shift_output(selected, shift);
  Ok((resized.scene, next_output, camera))
}

#[cfg(test)]
mod frame_shift_tests {
  use super::*;

  #[test]
  fn a_centred_grow_moves_placement_by_half_the_growth() {
    let old = WorldRect {
      x: 100.0,
      y: 100.0,
      width: 3600.0,
      height: 2338.0,
    };
    let new = WorldRect {
      x: 100.0 - (6765.0 - 3600.0) / 2.0,
      y: 100.0 - (4894.0 - 2338.0) / 2.0,
      width: 6765.0,
      height: 4894.0,
    };
    let shift = origin_shift(old, new);
    let mut output = crate::screenshots::test_output_settings(3600, 2338);
    output.crop_x = 0.0;
    output.crop_y = 0.0;
    output.image_x = 0.0;
    output.image_y = 0.0;
    shift_output(&mut output, shift);
    // The layer stays where it was on screen: half the growth now sits
    // before it on each axis.
    assert_eq!(output.crop_x, (6765.0 - 3600.0) / 2.0);
    assert_eq!(output.crop_y, (4894.0 - 2338.0) / 2.0);
    assert_eq!(output.image_x, output.crop_x);
    assert_eq!(output.image_y, output.crop_y);
  }

  #[test]
  fn a_far_edge_grow_leaves_placement_alone() {
    let old = WorldRect {
      x: 0.0,
      y: 0.0,
      width: 100.0,
      height: 100.0,
    };
    let new = WorldRect {
      x: 0.0,
      y: 0.0,
      width: 150.0,
      height: 100.0,
    };
    let mut output = crate::screenshots::test_output_settings(100, 100);
    output.crop_x = 10.0;
    output.image_x = 5.0;
    shift_output(&mut output, origin_shift(old, new));
    assert_eq!(output.crop_x, 10.0);
    assert_eq!(output.image_x, 5.0);
  }
}
