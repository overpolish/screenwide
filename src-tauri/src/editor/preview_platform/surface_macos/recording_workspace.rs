// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::ffi::{
  screenwide_preview_surface_present_recording_workspace,
  screenwide_preview_surface_redraw_workspace,
};
use super::native_types::RecordingWorkspaceLayer;
use super::workspace_scene::{LayerPicture, StagedLayer};
use super::RecordingPreviewSurface;
use crate::editor::cursor_effects::{GpuArtwork, GpuCursor};
use crate::editor::{media_preview, CameraOverlaySettings};
use crate::screenshots::{ScreenshotOutputSettings, StillOverlay};

impl RecordingPreviewSurface {
  /// Presents a retained recording scene with explicit per-layer placements.
  /// Unlike screenshot layers, recording panes are not implicitly coincident.
  pub(in crate::editor) fn present_recording_workspace(
    &self,
    layers: &[RecordingWorkspaceLayer<'_>],
    artworks: Option<&[GpuArtwork]>,
  ) -> Result<bool, String> {
    let picture = |image: Option<_>, pixels: Option<(*mut std::ffi::c_void, _)>| {
      image
        .map(LayerPicture::Image)
        .or_else(|| pixels.map(|(pixels, _)| LayerPicture::Pixels(pixels)))
    };
    let staged = layers
      .iter()
      .map(|layer| {
        Ok(StagedLayer {
          pane_index: layer.pane_index,
          layer_id: layer.pane_index,
          source_token: layer.source_token,
          source: picture(layer.source, layer.source_pixels)
            .ok_or_else(|| "Recording workspace layer has no source".to_owned())?,
          // A recording's redactions carry the fills held from their clips'
          // first frames, so all a frame adds is how large its source is.
          redaction_picture: None,
          settings: &layer.settings,
          seconds: layer.seconds,
          cursor: layer.cursor.map(|cursor| GpuCursor {
            clip_at_video_edge: layer.clip_cursor_at_video_edge,
            ..cursor
          }),
          keyboard: layer.keyboard,
          camera: picture(layer.camera, layer.camera_pixels),
          overlay: layer.overlay,
          foreground_only: layer.foreground_only,
          // The halo is set on the scene by `redraw_annotation_hover`.
          hover: None,
        })
      })
      .collect::<Result<Vec<_>, String>>()?;
    self.scene.stage(&staged, artworks)?;
    let panes = layers
      .iter()
      .map(|layer| layer.pane_index)
      .collect::<Vec<_>>();
    let placements = layers
      .iter()
      .map(|layer| layer.placement)
      .collect::<Vec<_>>();
    Ok(unsafe {
      screenwide_preview_surface_present_recording_workspace(
        self.handle,
        panes.as_ptr(),
        placements.as_ptr(),
        panes.len().try_into().unwrap_or(u32::MAX),
      ) != 0
    })
  }

  /// Rebuilds the retained layers' canvases over their resident pictures.
  /// This keeps crop/output transitions in the same native draw as the OSC
  /// without asking the still decoder for identical source pixels.
  pub(crate) fn recompose_recording_workspace(
    &self,
    panes: &[(u32, &ScreenshotOutputSettings)],
    baked_camera: Option<(CameraOverlaySettings, bool, bool)>,
  ) -> Result<bool, String> {
    // The incoming settings are the semantic source of truth once there is no
    // active native gesture, so each canvas's size and uniforms change
    // together: reusing a pre-undo size would stretch the restored pixels.
    for (pane_index, settings) in panes {
      if !self.scene.update_canvas(*pane_index, settings)? {
        return Ok(false);
      }
    }
    let Some((settings, drop_shadow, camera_on_top)) = baked_camera else {
      return Ok(true);
    };
    let Some((_, screen)) = panes.iter().find(|(index, _)| *index == 0) else {
      return Ok(false);
    };
    let Some((camera_width, camera_height)) = self.scene.camera_source_size(0) else {
      return Ok(false);
    };
    let geometry = media_preview::bake_geometry(media_preview::BakedVideoExportOptions {
      camera_drop_shadow: drop_shadow,
      camera_height,
      camera_width,
      overlay: settings,
      screen_height: screen.height,
      screen_width: screen.width,
      video: media_preview::VideoExportOptions {
        compression: 0,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
    })?;
    let overlay = StillOverlay {
      camera_crop_x: geometry.crop_x,
      camera_crop_y: geometry.crop_y,
      camera_crop_width: geometry.crop_width,
      camera_crop_height: geometry.crop_height,
      camera_frame_x: geometry.frame_x,
      camera_frame_y: geometry.frame_y,
      camera_frame_width: geometry.frame_width,
      camera_frame_height: geometry.frame_height,
      camera_radius: geometry.radius,
      camera_source_width: camera_width,
      camera_source_height: camera_height,
      camera_drop_shadow: u32::from(drop_shadow),
      camera_on_top: u32::from(camera_on_top),
      ..StillOverlay::default()
    };
    Ok(self.scene.update_camera(0, &overlay))
  }

  /// Moves the hover halo on the retained recording scene and redraws it.
  /// The halo is the one piece of annotation state the pointer changes
  /// without the document changing, so it is set on the scene rather than
  /// sent round through a fresh composition - which a recording cannot do
  /// from annotations alone, having a decoded frame behind them.
  ///
  /// `hover` is the pane, the annotation's place in its own list and the
  /// halo's width; `None` clears.
  pub(crate) fn redraw_annotation_hover(&self, hover: Option<(u32, usize, f32)>) -> bool {
    let moved = self.scene.set_hover(
      hover.map_or(0, |(pane, _, _)| pane),
      hover.map(|(_, index, width)| (index, width)),
    );
    moved && self.redraw_recording_workspace()
  }

  /// Presents the retained recording sources after a uniform-only edit.
  pub(crate) fn redraw_recording_workspace(&self) -> bool {
    unsafe { screenwide_preview_surface_redraw_workspace(self.handle) != 0 }
  }
}
