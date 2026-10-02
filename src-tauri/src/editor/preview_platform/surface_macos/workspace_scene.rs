// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The macOS editor workspace's retained scene, drawn by the shared
//! compositor into the workspace drawable.
//!
//! Decoders and the screenshot preview stage layers here; Objective-C keeps
//! the layer, the input and the present, and calls back in to draw on pan and
//! zoom, to move the scene with a native gesture before React has caught up,
//! and to read where the shortcut strip is.

mod draw;
mod exports;
mod gestures;
mod layers;
mod loupe;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use self::layers::{LayerCamera, SceneLayer, ScenePicture};
pub(crate) use self::layers::{LayerPicture, StagedLayer};
use crate::editor::cursor_effects::GpuArtwork;
use crate::editor::preview_platform::annotation_gpu::SourceAnnotations;
use crate::editor::preview_platform::compositor::{
  CanvasGeometry, Compositor, CursorArtwork, NativeCursors,
};
use crate::screenshots::{ScreenshotOutputSettings, StillOverlay};

struct SceneState {
  compositor: Compositor,
  loupe: loupe::Loupe,
  layers: Vec<SceneLayer>,
  /// The layers as a native gesture found them, and whether an update has
  /// rewritten the scene since.
  resize: Option<(Vec<SceneLayer>, bool)>,
  sources: HashMap<u64, Arc<ScenePicture>>,
}

pub(crate) struct WorkspaceScene {
  state: Mutex<SceneState>,
}

impl WorkspaceScene {
  pub(super) fn new() -> Result<Self, String> {
    let gpu = crate::gpu::shared()?;
    Ok(Self {
      state: Mutex::new(SceneState {
        compositor: Compositor::new(gpu, NativeCursors::none())?,
        loupe: loupe::Loupe::new(gpu),
        layers: Vec::new(),
        resize: None,
        sources: HashMap::new(),
      }),
    })
  }

  fn state(&self) -> Result<MutexGuard<'_, SceneState>, String> {
    self
      .state
      .lock()
      .map_err(|_| "The workspace scene is poisoned".to_owned())
  }

  /// Replaces the scene with `layers`. While a native gesture has rewritten
  /// the scene, a late frame would put back geometry from an earlier pointer
  /// sample, so it is accepted without being staged. Before that, as during a
  /// plain move that only opened the session so Option can auto-fit later,
  /// the decoder's frames are the only source of the moved pixels.
  pub(super) fn stage(
    &self,
    layers: &[StagedLayer<'_>],
    artworks: Option<&[GpuArtwork]>,
  ) -> Result<(), String> {
    let mut state = self.state()?;
    if state.resize.as_ref().is_some_and(|(_, applied)| *applied) {
      return Ok(());
    }
    if let Some(artworks) = artworks.filter(|artworks| !artworks.is_empty()) {
      let artworks = artworks.iter().map(CursorArtwork::from).collect::<Vec<_>>();
      state.compositor.use_cursor_artworks(&artworks)?;
    }
    let no_overlay = StillOverlay::default();
    let mut staged = Vec::with_capacity(layers.len());
    for layer in layers {
      let source = match state.sources.get(&layer.source_token) {
        Some(source) if source.texture.size == layer.source.size() => Arc::clone(source),
        _ => {
          let source = Arc::new(ScenePicture::new(&state.compositor, layer.source)?);
          state
            .sources
            .insert(layer.source_token, Arc::clone(&source));
          source
        }
      };
      let size = source.texture.size;
      let camera = layer
        .camera
        .map(|picture| {
          let picture = ScenePicture::new(&state.compositor, picture)?;
          let overlay = layer.overlay.unwrap_or(&no_overlay);
          Ok::<_, String>(LayerCamera::new(overlay, Some(Arc::new(picture))))
        })
        .transpose()?;
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
        cursor: layer.cursor,
        keyboard: layer.keyboard,
        camera,
        foreground_only: layer.foreground_only,
        hover: layer.hover,
      });
    }
    state
      .sources
      .retain(|token, _| staged.iter().any(|layer| layer.source_token == *token));
    state.layers = staged;
    Ok(())
  }

  /// The size of the camera frame `pane_index` carries, once it is resident.
  pub(super) fn camera_source_size(&self, pane_index: u32) -> Option<(u32, u32)> {
    let state = self.state().ok()?;
    let layer = state
      .layers
      .iter()
      .find(|layer| layer.pane_index == pane_index)?;
    let picture = layer.camera.as_ref()?.picture.as_ref()?;
    Some(picture.texture.size)
  }

  /// Rebuilds `pane_index`'s canvas from `settings` over its resident
  /// picture. The cursor is in canvas pixels, so it is carried through the
  /// same change of image placement as the clip.
  pub(super) fn update_canvas(
    &self,
    pane_index: u32,
    settings: &ScreenshotOutputSettings,
  ) -> Result<bool, String> {
    let mut state = self.state()?;
    let Some(layer) = state
      .layers
      .iter_mut()
      .find(|layer| layer.pane_index == pane_index)
    else {
      return Ok(false);
    };
    let geometry = CanvasGeometry::of(layer.source.texture.size, settings)?;
    let (from, to) = (layer.geometry.placement, geometry.placement);
    if let Some(cursor) = layer
      .cursor
      .as_mut()
      .filter(|_| from.image_width > 0 && from.image_height > 0)
    {
      let sx = f64::from(to.image_width) / f64::from(from.image_width);
      let sy = f64::from(to.image_height) / f64::from(from.image_height);
      cursor.x = (to.image_x + (f64::from(cursor.x) - from.image_x) * sx) as f32;
      cursor.y = (to.image_y + (f64::from(cursor.y) - from.image_y) * sy) as f32;
      cursor.width *= sx as f32;
      cursor.height *= sy as f32;
      cursor.hotspot_x *= sx as f32;
      cursor.hotspot_y *= sy as f32;
      cursor.blur_delta_x *= sx as f32;
      cursor.blur_delta_y *= sy as f32;
    }
    layer.geometry = geometry;
    layer.settings = Arc::new(settings.clone());
    Ok(true)
  }

  /// Moves `pane_index`'s camera bubble, whose frame must be resident.
  pub(super) fn update_camera(&self, pane_index: u32, overlay: &StillOverlay) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    let Some(camera) = state
      .layers
      .iter_mut()
      .find(|layer| layer.pane_index == pane_index)
      .and_then(|layer| layer.camera.as_mut())
      .filter(|camera| camera.picture.is_some())
    else {
      return false;
    };
    *camera = LayerCamera::new(overlay, camera.picture.take());
    true
  }

  /// Puts the hover halo on the annotation at `index` in `pane_index`'s own
  /// list, with its width in canvas pixels, and takes it off every other.
  /// `None` only clears.
  pub(super) fn set_hover(&self, pane_index: u32, hover: Option<(usize, f32)>) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    let mut found = false;
    for layer in &mut state.layers {
      found |= layer.pane_index == pane_index;
      layer.hover = hover.filter(|_| layer.pane_index == pane_index);
    }
    found
  }
}
