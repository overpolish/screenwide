// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The editor workspace's retained scene, drawn by the shared compositor
//! into the workspace drawable.
//!
//! Decoders and the screenshot preview stage layers here; the platform
//! surface keeps the drawable, the input and the present, and calls in to
//! draw on pan and zoom, to move the scene with a native gesture before React
//! has caught up, and to read where the shortcut strip is.

mod draw;
mod effects;
// Windows drives Frame resizes and radius drags through React rather than
// these native gestures.
#[cfg_attr(target_os = "windows", allow(dead_code))]
mod gestures;
mod layers;
mod loupe;
mod overlay_bounds;
mod picture;
mod recording;
mod stage;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

pub use self::draw::NativeWorkspacePlacement;
#[cfg_attr(target_os = "windows", allow(unused_imports))]
pub use self::gestures::FrameChange;
pub use self::layers::StagedLayer;
use self::layers::{LayerCamera, SceneLayer};
#[cfg(target_os = "windows")]
pub use self::picture::DecodedTexture;
pub use self::picture::LayerPicture;
use self::picture::ScenePicture;
pub use self::recording::{baked_camera_overlay, RecordingWorkspaceLayer};
use crate::backdrop::Backdrop;
use screenwide_compositor::compositor::still_overlay::StillOverlay;
use screenwide_compositor::compositor::{CanvasGeometry, Compositor};
use screenwide_output::ScreenshotOutputSettings;
use screenwide_text::engine::typing::TypingMarks;

struct SceneState {
  compositor: Compositor,
  backdrop: Backdrop,
  loupe: loupe::Loupe,
  layers: Vec<SceneLayer>,
  /// The layers as a native gesture found them, and whether an update has
  /// rewritten the scene since.
  resize: Option<(Vec<SceneLayer>, bool)>,
  sources: HashMap<u64, Arc<ScenePicture>>,
  /// The camera's effects, drawn on every camera frame the scene holds.
  effects: screenwide_export::camera_effects::preview::PreviewEffects,
  /// The box being typed into: its layer, its place in that layer's list,
  /// and the caret and selection drawn into its type. Kept on the scene
  /// rather than on a layer, so the restaging every typed change causes
  /// keeps it.
  typing: Option<(u32, usize, TypingMarks)>,
  /// Counts every time the scene may have changed; the retained picture is
  /// only shown again while it is the revision that picture was drawn at.
  revision: u64,
  /// The scene as last composed, without the selection chrome drawn over it,
  /// so a redraw of the chrome alone does not compose every layer again.
  picture: Option<draw::RetainedPicture>,
}

pub struct WorkspaceScene {
  state: Mutex<SceneState>,
}

impl WorkspaceScene {
  pub fn new() -> Result<Self, String> {
    let gpu = screenwide_gpu::shared()?;
    let compositor = Compositor::new(gpu)?;
    // Drawn as it is edited: a set of kinds new to it is compiled on another
    // thread while the frames go on with a variant that holds it.
    compositor.compile_annotation_kinds_in_background();
    Ok(Self {
      state: Mutex::new(SceneState {
        compositor,
        backdrop: Backdrop::new(gpu),
        loupe: loupe::Loupe::new(gpu),
        layers: Vec::new(),
        resize: None,
        sources: HashMap::new(),
        effects: Default::default(),
        typing: None,
        revision: 0,
        picture: None,
      }),
    })
  }

  /// The scene to change. Every hold counts as a change, so a picture drawn
  /// before it is never shown again.
  fn state(&self) -> Result<MutexGuard<'_, SceneState>, String> {
    let mut state = self.read()?;
    state.revision = state.revision.wrapping_add(1);
    Ok(state)
  }

  /// The scene to read or to draw, which leaves the retained picture current.
  fn read(&self) -> Result<MutexGuard<'_, SceneState>, String> {
    self
      .state
      .lock()
      .map_err(|_| "The workspace scene is poisoned".to_owned())
  }

  /// Draws the caret and selection `typing` holds into its box from now on,
  /// or none with `None`. Answers whether that changes what is drawn.
  pub fn set_typing(&self, typing: Option<(u32, usize, TypingMarks)>) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    let changed = state.typing != typing;
    state.typing = typing;
    changed
  }

  /// The size of the camera frame `pane_index` carries, once it is resident.
  pub fn camera_source_size(&self, pane_index: u32) -> Option<(u32, u32)> {
    let state = self.read().ok()?;
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
  pub fn update_canvas(
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
    if let Some(cursor) = layer.cursor.as_mut() {
      *cursor =
        screenwide_compositor::compositor::cursor_placement::carried_cursor(*cursor, &from, &to);
    }
    layer.geometry = geometry;
    layer.settings = Arc::new(settings.clone());
    Ok(true)
  }

  /// Moves `pane_index`'s camera bubble, whose frame must be resident.
  pub fn update_camera(&self, pane_index: u32, overlay: &StillOverlay) -> bool {
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
    let composed = LayerCamera::new(overlay, camera.picture.take());
    *camera = LayerCamera {
      composition: camera.composition.take(),
      hover: camera.hover,
      matte: camera.matte.take(),
      ..composed
    };
    true
  }

  /// Puts the hover halo on the annotation at `index` in `pane_index`'s own
  /// list, with its width in canvas pixels, and takes it off every other.
  /// `None` only clears. With no layer of its own, the camera's annotations
  /// are the ones composed into the camera frame a layer carries.
  pub fn set_hover(&self, pane_index: u32, hover: Option<(usize, f32)>) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    let own = state
      .layers
      .iter()
      .any(|layer| layer.pane_index == pane_index);
    let mut found = own;
    for layer in &mut state.layers {
      layer.hover = hover.filter(|_| layer.pane_index == pane_index);
      if let Some(camera) = layer.camera.as_mut() {
        let composed = !own && pane_index == 1 && camera.composition.is_some();
        found |= composed;
        camera.hover = hover.filter(|_| composed);
      }
    }
    found
  }
}
