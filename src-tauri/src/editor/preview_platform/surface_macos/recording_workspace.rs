// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::ffi::{
  screenwide_preview_surface_present_recording_workspace,
  screenwide_preview_surface_redraw_workspace,
};
use super::RecordingPreviewSurface;
use crate::editor::preview_platform::workspace_scene::RecordingWorkspaceLayer;
use screenwide_cursor_effects::GpuArtwork;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::CameraOverlaySettings;

impl RecordingPreviewSurface {
  /// Presents a retained recording scene with explicit per-layer placements.
  /// Unlike screenshot layers, recording panes are not implicitly coincident.
  pub(in crate::editor) fn present_recording_workspace(
    &self,
    layers: &[RecordingWorkspaceLayer<'_>],
    artworks: Option<&[GpuArtwork]>,
  ) -> Result<bool, String> {
    if let Some(artworks) = artworks {
      self.scene.use_cursor_artworks(artworks)?;
    }
    self.scene.stage_recording(layers)?;
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
    self.scene.recompose_recording(panes, baked_camera)
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

  /// Draws the camera with `live` from now on, and draws the scene again
  /// over the frames it holds.
  pub(crate) fn set_camera_effects(
    &self,
    live: Option<std::sync::Arc<screenwide_export::camera_effects::live::LiveCameraEffects>>,
  ) -> Result<(), String> {
    self.scene.set_camera_effects(live)?;
    self.redraw_recording_workspace();
    Ok(())
  }
}
