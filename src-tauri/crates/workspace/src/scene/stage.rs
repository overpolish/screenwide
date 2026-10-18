// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Staging: the layers a present hands over replace the scene's, their
//! pictures kept resident by token.

use std::sync::Arc;

use screenwide_imaging::CapturedImage;
use screenwide_output::ScreenshotOutputSettings;

use super::layers::{LayerCamera, SceneLayer, StagedLayer};
use super::picture::{LayerPicture, ScenePicture};
use super::WorkspaceScene;
use screenwide_compositor::annotation_gpu::SourceAnnotations;
use screenwide_compositor::compositor::still_overlay::StillOverlay;
use screenwide_compositor::compositor::{CameraComposition, CanvasGeometry};

impl WorkspaceScene {
  /// Stages every screenshot item as one layer of one pane, the first the
  /// whole canvas and the rest laid over it. `hover` is the hovered
  /// annotation's halo: its item, its place in that item's list, and its
  /// width in canvas pixels; it never reaches the export.
  #[allow(clippy::type_complexity)]
  pub fn stage_screenshot(
    &self,
    layers: &[(u64, &CapturedImage, ScreenshotOutputSettings)],
    hover: Option<(u64, usize, f32)>,
  ) -> Result<(), String> {
    let staged = layers
      .iter()
      .enumerate()
      .map(|(index, (source_token, source, settings))| StagedLayer {
        pane_index: 0,
        // Screenshot selection and gesture events address layers by their
        // workspace order. `source_token` remains the independent cache key;
        // using it as the layer identity prevents the crop magnifier from
        // resolving the selected retained source.
        layer_id: u32::try_from(index).unwrap_or(u32::MAX - 1),
        source_token: *source_token,
        source: LayerPicture::Image(source),
        redaction_picture: Some(source),
        settings,
        seconds: 0.0,
        cursor: None,
        keyboard: None,
        captions: None,
        camera: None,
        overlay: None,
        camera_settings: None,
        foreground_only: index > 0,
        source_is_camera: false,
        hover: hover
          .filter(|(layer, _, _)| layer == source_token)
          .map(|(_, index, width)| (index, width)),
      })
      .collect::<Vec<_>>();
    self.stage(&staged)
  }

  /// Replaces the scene with `layers`. While a native gesture has rewritten
  /// the scene, a late frame would put back geometry from an earlier pointer
  /// sample, so it is accepted without being staged. Before that, as during a
  /// plain move that only opened the session so Option can auto-fit later,
  /// the decoder's frames are the only source of the moved pixels.
  pub fn stage(&self, layers: &[StagedLayer<'_>]) -> Result<(), String> {
    let mut state = self.state()?;
    if state.resize.as_ref().is_some_and(|(_, applied)| *applied) {
      return Ok(());
    }
    let state = &mut *state;
    let no_overlay = StillOverlay::default();
    let mut staged = Vec::with_capacity(layers.len());
    for layer in layers {
      let resident = state
        .layers
        .iter()
        .find(|resident| resident.pane_index == layer.pane_index);
      let held = resident.filter(|_| layer.source.is_held());
      let cached = state
        .sources
        .get(&layer.source_token)
        .filter(|source| source.texture.size == layer.source.size() || source.takes(layer.source));
      // A picture the pane already shows can take this one too: held as it
      // is, or refilled with a decoded frame of its size, whatever the token.
      let source = match cached.or_else(|| {
        resident
          .map(|resident| &resident.source)
          .filter(|source| source.takes(layer.source))
      }) {
        Some(source) => {
          source.refill(&state.compositor, layer.source)?;
          Arc::clone(source)
        }
        // A held picture the scene never had is nothing to draw yet.
        None if layer.source.is_held() => continue,
        None => Arc::new(ScenePicture::new(&state.compositor, layer.source)?),
      };
      let size = source.texture.size;
      let camera = match layer.camera {
        None => None,
        // A held camera keeps its frame, and its place, as the scene holds
        // them; only what is composed into it follows the settings.
        Some(picture) if picture.is_held() => resident
          .and_then(|resident| resident.camera.clone())
          .map(|mut camera| {
            if let (Some((settings, source)), Some(frame)) =
              (layer.camera_settings, camera.picture.as_ref())
            {
              camera.composition =
                CameraComposition::new(settings, source, frame.texture.size)?.map(Arc::new);
            }
            camera.matte = state.effects.matte_at(layer.seconds);
            Ok::<_, String>(camera)
          })
          .transpose()?,
        Some(picture) => {
          let resident_camera =
            resident.and_then(|resident| resident.camera.as_ref()?.picture.clone());
          let picture = match resident_camera.filter(|resident| resident.takes(picture)) {
            Some(resident) => {
              resident.refill(&state.compositor, picture)?;
              resident
            }
            None => Arc::new(ScenePicture::new(&state.compositor, picture)?),
          };
          let overlay = layer.overlay.unwrap_or(&no_overlay);
          let composition = layer
            .camera_settings
            .map(|(settings, source)| {
              CameraComposition::new(settings, source, picture.texture.size)
            })
            .transpose()?
            .flatten();
          let mut camera = LayerCamera::new(overlay, Some(picture));
          camera.composition = composition.map(Arc::new);
          camera.matte = state.effects.matte_at(layer.seconds);
          Some(camera)
        }
      };
      let source_matte = layer
        .source_is_camera
        .then(|| state.effects.matte_at(layer.seconds))
        .flatten();
      staged.push(SceneLayer {
        pane_index: layer.pane_index,
        layer_id: layer.layer_id,
        source_token: layer.source_token,
        geometry: CanvasGeometry::of(size, layer.settings)?,
        annotations: Arc::new(SourceAnnotations::new(
          &layer.settings.annotations,
          size,
          layer.settings,
          layer.redaction_picture,
        )),
        settings: Arc::new(layer.settings.clone()),
        source,
        seconds: layer.seconds,
        // A picture held over from the last frame keeps what that frame
        // carried over it.
        cursor: held.map_or(layer.cursor, |resident| resident.cursor),
        keyboard: held.map_or(layer.keyboard, |resident| resident.keyboard),
        captions: layer.captions.clone(),
        camera,
        foreground_only: layer.foreground_only,
        source_is_camera: layer.source_is_camera,
        source_matte,
        hover: layer.hover,
      });
    }
    state.sources.clear();
    for layer in &staged {
      state
        .sources
        .insert(layer.source_token, Arc::clone(&layer.source));
    }
    state.layers = staged;
    Ok(())
  }
}
