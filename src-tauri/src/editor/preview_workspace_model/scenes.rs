// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

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
