// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording workspace, drawn through the shared scene as on macOS: the
//! player stages each moment's decoded frames, and the edits that leave the
//! frames alone redraw what the scene holds.

use super::*;
use crate::editor::preview_platform::RecordingWorkspaceLayer;

/// A decoded frame as the decoder holds it: its texture, the subresource the
/// frame is in, and its size.
pub(crate) type DecodedTexture<'a> = (&'a ID3D11Texture2D, u32, (u32, u32));

impl RecordingPreviewSurface {
  /// Presents a recording's layers, each in its own pane. `Ok(false)` means
  /// layout has not made a layer's pane yet.
  pub(in crate::editor) fn present_recording_workspace(
    &self,
    layers: &[RecordingWorkspaceLayer<'_>],
  ) -> Result<bool, String> {
    let Ok(mut state) = self.inner.state.lock() else {
      return Ok(false);
    };
    if layers.iter().any(|layer| {
      state
        .panes
        .get(layer.pane_index as usize)
        .is_none_or(Option::is_none)
    }) {
      return Ok(false);
    }
    self.inner.gpu.workspace.scene.stage_recording(layers)?;
    for layer in layers {
      let Some(pane) = state.panes[layer.pane_index as usize].as_mut() else {
        continue;
      };
      let canvas = (layer.settings.width, layer.settings.height);
      if pane.content_size != canvas {
        pane.content_size = canvas;
        pane.selection_stale = true;
      }
      pane.settings = Some(layer.settings.clone());
      pane.captions = layer.captions.clone();
      pane.source_token = None;
    }
    state.workspace.panes = self
      .inner
      .gpu
      .workspace
      .scene
      .layer_panes()
      .into_iter()
      .map(|pane| pane as usize)
      .collect();
    state.workspace.recording = true;
    present_workspace(&self.inner, &mut state);
    redraw_stale_selection(&self.inner, &mut state);
    Ok(true)
  }

  /// Rebuilds the retained layers' canvases over their resident pictures.
  /// This keeps crop/output transitions in the same native draw as the OSC
  /// without asking the still decoder for identical source pixels.
  pub(crate) fn recompose_recording_workspace(
    &self,
    panes: &[(u32, &ScreenshotOutputSettings)],
    baked_camera: Option<(screenwide_scenes::CameraOverlaySettings, bool, bool)>,
  ) -> Result<bool, String> {
    let Ok(mut state) = self.inner.state.lock() else {
      return Ok(false);
    };
    if !self
      .inner
      .gpu
      .workspace
      .scene
      .recompose_recording(panes, baked_camera)?
    {
      return Ok(false);
    }
    for (index, settings) in panes {
      let Some(pane) = state
        .panes
        .get_mut(*index as usize)
        .and_then(Option::as_mut)
      else {
        continue;
      };
      let canvas = (settings.width, settings.height);
      if pane.content_size != canvas {
        pane.content_size = canvas;
        pane.selection_stale = true;
      }
      // The settings arrive without the annotations, which only a frame
      // resolves; the pane keeps the ones its frame carried.
      let annotations = pane
        .settings
        .take()
        .map(|settings| settings.annotations)
        .unwrap_or_default();
      pane.settings = Some(ScreenshotOutputSettings {
        annotations,
        ..(*settings).clone()
      });
    }
    Ok(true)
  }

  /// Presents the retained recording sources after a uniform-only edit.
  pub(crate) fn redraw_recording_workspace(&self) -> bool {
    let Ok(mut state) = self.inner.state.lock() else {
      return false;
    };
    present_workspace(&self.inner, &mut state);
    redraw_stale_selection(&self.inner, &mut state);
    !state.workspace.panes.is_empty()
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
    let moved = self.inner.gpu.workspace.scene.set_hover(
      hover.map_or(0, |(pane, _, _)| pane),
      hover.map(|(_, index, width)| (index, width)),
    );
    moved && self.redraw_recording_workspace()
  }

  /// Draws the camera with `live` from now on, and draws the scene again
  /// over the frames it holds.
  pub(crate) fn set_camera_effects(
    &self,
    live: Option<std::sync::Arc<screenwide_export::camera_effects::live::LiveCameraEffects>>,
  ) -> Result<(), String> {
    self.inner.gpu.workspace.scene.set_camera_effects(live)?;
    self.redraw_recording_workspace();
    Ok(())
  }

  /// Draws `annotations` over the frame pane `index` already holds, in place
  /// of the ones it was staged with. This is what an arrow drag needs per
  /// pointer sample: the picture has not changed, only the annotations over
  /// it, so asking the decoder for the frame again would put a seek between
  /// the hand and the arrow.
  ///
  /// `source` is the full-resolution grid the annotations are authored in;
  /// they are moved onto the decoded frame's own grid here, as a live present
  /// does. `camera` is a baked camera's own settings, carrying the
  /// annotations composed into its frame, and its source size.
  pub(crate) fn redraw_recording_annotations(
    &self,
    index: u32,
    annotations: &[screenwide_annotations::Annotation],
    source: (u32, u32),
    camera: Option<(&ScreenshotOutputSettings, (u32, u32))>,
  ) -> bool {
    let Ok(mut state) = self.inner.state.lock() else {
      return false;
    };
    let scene = &self.inner.gpu.workspace.scene;
    let (Some(mut settings), Some(decoded)) = (
      state
        .panes
        .get(index as usize)
        .and_then(Option::as_ref)
        .and_then(|pane| pane.settings.clone()),
      scene.source_size(index),
    ) else {
      return false;
    };
    settings.annotations = annotations.to_vec();
    let output_width = settings.width;
    crate::editor::recording_preview_player::annotation_preview::remap_source(
      &mut settings,
      source,
      decoded,
      output_width,
    );
    if !scene
      .update_annotations(index, &settings, camera)
      .unwrap_or(false)
    {
      return false;
    }
    if let Some(pane) = state.panes.get_mut(index as usize).and_then(Option::as_mut) {
      pane.settings = Some(settings);
    }
    present_workspace(&self.inner, &mut state);
    true
  }
}
