// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording's layers as a player hands them over: the screen and, in
//! Separate files, the camera, each drawn from a decoded frame, with a camera
//! baked into the screen's canvas in One video.

use std::sync::Arc;

use super::{LayerPicture, NativeWorkspacePlacement, StagedLayer, WorkspaceScene};
#[cfg(target_os = "windows")]
use screenwide_compositor::annotation_gpu::SourceAnnotations;
use screenwide_compositor::compositor::still_overlay::StillOverlay;
#[cfg(target_os = "windows")]
use screenwide_compositor::compositor::CameraComposition;
use screenwide_cursor_effects::GpuCursor;
use screenwide_export::media_preview;
use screenwide_keyboard_effects::KeyboardOverlay;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::CameraOverlaySettings;

/// Input for one layer of the retained recording workspace: its decoded frame,
/// and a camera frame composed into it beside the cursor and the shortcut
/// strip.
pub(crate) struct RecordingWorkspaceLayer<'a> {
  pub pane_index: u32,
  /// The cache key of the picture: a new token is a new picture.
  pub source_token: u64,
  pub source: LayerPicture<'a>,
  pub settings: ScreenshotOutputSettings,
  /// Where the surface draws the layer, where its caller places it. Windows
  /// places every layer in its pane's box instead.
  #[cfg_attr(target_os = "windows", allow(dead_code))]
  pub placement: NativeWorkspacePlacement,
  pub seconds: f64,
  pub cursor: Option<GpuCursor>,
  pub keyboard: Option<KeyboardOverlay>,
  /// The captions drawn over the canvas, and the moment of the edited video
  /// they are drawn at.
  pub captions: Option<(
    Arc<screenwide_text::captions::CaptionTrack>,
    screenwide_text::captions::CaptionMoment,
  )>,
  pub camera: Option<LayerPicture<'a>>,
  pub overlay: Option<&'a StillOverlay>,
  /// In One video, the camera's own settings, carrying the annotations drawn
  /// into its frame, and the camera's source size, which they are placed in.
  pub camera_settings: Option<(&'a ScreenshotOutputSettings, (u32, u32))>,
  pub clip_cursor_at_video_edge: bool,
  pub foreground_only: bool,
  /// Whether the layer's own picture is the camera, which the camera's
  /// effects draw on.
  pub source_is_camera: bool,
}

impl WorkspaceScene {
  /// Stages a recording's layers, each its own pane's.
  pub(in crate::editor::preview_platform) fn stage_recording(
    &self,
    layers: &[RecordingWorkspaceLayer<'_>],
  ) -> Result<(), String> {
    let staged = layers
      .iter()
      .map(|layer| StagedLayer {
        pane_index: layer.pane_index,
        layer_id: layer.pane_index,
        source_token: layer.source_token,
        source: layer.source,
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
        captions: layer.captions.clone(),
        camera: layer.camera,
        overlay: layer.overlay,
        camera_settings: layer.camera_settings,
        foreground_only: layer.foreground_only,
        source_is_camera: layer.source_is_camera,
        // The halo is set on the scene by `set_hover`.
        hover: None,
      })
      .collect::<Vec<_>>();
    self.stage(&staged)
  }

  /// Rebuilds the retained layers' canvases over their resident pictures:
  /// each listed pane's from its settings, and a camera baked into the
  /// screen's from where `baked_camera`, its overlay, drop shadow and
  /// whether it sits in front, puts it. This keeps crop and output
  /// transitions in one draw with the chrome, without asking the decoder for
  /// the same source pixels. `false` where a pane or its camera frame is not
  /// resident.
  pub(in crate::editor::preview_platform) fn recompose_recording(
    &self,
    panes: &[(u32, &ScreenshotOutputSettings)],
    baked_camera: Option<(CameraOverlaySettings, bool, bool)>,
  ) -> Result<bool, String> {
    // The incoming settings are the semantic source of truth once there is no
    // active native gesture, so each canvas's size and uniforms change
    // together: reusing a pre-undo size would stretch the restored pixels.
    for (pane_index, settings) in panes {
      if !self.update_canvas(*pane_index, settings)? {
        return Ok(false);
      }
    }
    let Some((settings, drop_shadow, camera_on_top)) = baked_camera else {
      return Ok(true);
    };
    let Some((_, screen)) = panes.iter().find(|(index, _)| *index == 0) else {
      return Ok(false);
    };
    let Some(camera) = self.camera_source_size(0) else {
      return Ok(false);
    };
    let overlay = baked_camera_overlay(camera, screen, settings, drop_shadow, camera_on_top)?;
    Ok(self.update_camera(0, &overlay))
  }

  /// The pane each layer the scene holds is drawn in, in the scene's order.
  #[cfg(target_os = "windows")]
  pub(in crate::editor::preview_platform) fn layer_panes(&self) -> Vec<u32> {
    self.read().map_or_else(
      |_| Vec::new(),
      |state| state.layers.iter().map(|layer| layer.pane_index).collect(),
    )
  }

  /// The size of the picture `pane_index`'s layer is drawn from, once it is
  /// resident: the grid its annotations are placed on.
  #[cfg(target_os = "windows")]
  pub(in crate::editor::preview_platform) fn source_size(
    &self,
    pane_index: u32,
  ) -> Option<(u32, u32)> {
    let state = self.read().ok()?;
    let layer = state
      .layers
      .iter()
      .find(|layer| layer.pane_index == pane_index)?;
    Some(layer.source.texture.size)
  }

  /// Draws `settings`' annotations, already on the resident picture's grid,
  /// over `pane_index`'s layer from now on, over the same picture: an arrow
  /// drag changes them every sample without the frame changing. `camera` is
  /// a baked camera's own settings, carrying the annotations composed into
  /// its frame, and its source size. `false` where the pane has no layer.
  #[cfg(target_os = "windows")]
  pub(in crate::editor::preview_platform) fn update_annotations(
    &self,
    pane_index: u32,
    settings: &ScreenshotOutputSettings,
    camera: Option<(&ScreenshotOutputSettings, (u32, u32))>,
  ) -> Result<bool, String> {
    let mut state = self.state()?;
    let Some(layer) = state
      .layers
      .iter_mut()
      .find(|layer| layer.pane_index == pane_index)
    else {
      return Ok(false);
    };
    layer.annotations = Arc::new(SourceAnnotations::new(
      &settings.annotations,
      layer.source.texture.size,
      settings,
      None,
    ));
    layer.settings = Arc::new(settings.clone());
    if let (Some((camera_settings, camera_source)), Some(baked)) = (camera, layer.camera.as_mut()) {
      if let Some(picture) = baked.picture.as_ref() {
        baked.composition =
          CameraComposition::new(camera_settings, camera_source, picture.texture.size)?
            .map(Arc::new);
      }
    }
    Ok(true)
  }
}

/// Where a camera frame `camera` pixels in size sits in the screen's canvas,
/// which `screen` describes, in One video: where `overlay` places it, with or
/// without its drop shadow, in front of the screen or behind it.
pub(crate) fn baked_camera_overlay(
  camera: (u32, u32),
  screen: &ScreenshotOutputSettings,
  overlay: CameraOverlaySettings,
  drop_shadow: bool,
  on_top: bool,
) -> Result<StillOverlay, String> {
  let geometry = media_preview::bake_geometry(media_preview::BakedVideoExportOptions {
    camera_drop_shadow: drop_shadow,
    camera_height: camera.1,
    camera_width: camera.0,
    overlay,
    screen_height: screen.height,
    screen_width: screen.width,
    video: media_preview::VideoExportOptions {
      compression: 0,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
  })?;
  Ok(StillOverlay {
    camera_crop_x: geometry.crop_x,
    camera_crop_y: geometry.crop_y,
    camera_crop_width: geometry.crop_width,
    camera_crop_height: geometry.crop_height,
    camera_frame_x: geometry.frame_x,
    camera_frame_y: geometry.frame_y,
    camera_frame_width: geometry.frame_width,
    camera_frame_height: geometry.frame_height,
    camera_radius: geometry.radius,
    camera_source_width: camera.0,
    camera_source_height: camera.1,
    camera_drop_shadow: u32::from(drop_shadow),
    camera_on_top: u32::from(on_top),
    ..StillOverlay::default()
  })
}
