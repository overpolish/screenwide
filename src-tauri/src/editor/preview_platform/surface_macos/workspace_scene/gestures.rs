// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native gestures move the retained scene directly, so the picture and the
//! OSC show the same pointer sample before React mirrors the edit. Every
//! update is absolute: it is measured from the scene as the gesture found it.

use super::{SceneLayer, SceneState, WorkspaceScene};

/// How far a Frame gesture has taken a canvas, as ratios of its size when the
/// gesture began: where the origin moved to, the new size, and how far the
/// selected layer itself moved inside it.
#[derive(Clone, Copy)]
pub(super) struct FrameChange {
  pub(super) origin: (f64, f64),
  pub(super) size: (f64, f64),
  pub(super) moved: (f64, f64),
}

impl FrameChange {
  fn valid(self) -> bool {
    [self.origin.0, self.origin.1, self.size.0, self.size.1]
      .iter()
      .all(|value| value.is_finite())
      && self.size.0 > 0.0
      && self.size.1 > 0.0
  }
}

/// `layer`'s canvas resized by `change`: its new size, how far its origin
/// moved, and how far the selected layer moved, in its old canvas pixels.
fn resized(layer: &SceneLayer, change: FrameChange) -> ((u32, u32), (f64, f64), (f64, f64)) {
  let old = (
    f64::from(layer.geometry.size.0.max(1)),
    f64::from(layer.geometry.size.1.max(1)),
  );
  let next = (
    (old.0 * change.size.0).round().max(1.0) as u32,
    (old.1 * change.size.1).round().max(1.0) as u32,
  );
  (
    next,
    (old.0 * change.origin.0, old.1 * change.origin.1),
    (old.0 * change.moved.0, old.1 * change.moved.1),
  )
}

/// Scales the canvas rounding with the canvas's shorter side.
fn scale_background_radius(layer: &mut SceneLayer, next: (u32, u32)) {
  let old = f64::from(layer.geometry.size.0.min(layer.geometry.size.1)).max(1.0);
  let next_shortest = f64::from(next.0.min(next.1));
  layer.geometry.background_radius =
    (f64::from(layer.geometry.background_radius) * next_shortest / old).round() as f32;
  layer.geometry.size = next;
}

impl SceneState {
  /// The layers a gesture update starts from: as the gesture found them, or
  /// the scene itself when no gesture holds a snapshot.
  fn gesture_base(&self) -> Vec<SceneLayer> {
    match &self.resize {
      Some((layers, _)) if !layers.is_empty() => layers.clone(),
      _ => self.layers.clone(),
    }
  }

  fn has_snapshot(&self) -> bool {
    self
      .resize
      .as_ref()
      .is_some_and(|(layers, _)| !layers.is_empty())
  }

  fn applied(&mut self, layers: Vec<SceneLayer>) {
    if let Some((_, applied)) = self.resize.as_mut() {
      *applied = true;
    }
    self.layers = layers;
  }
}

impl WorkspaceScene {
  pub(super) fn begin_resize(&self) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    state.resize = Some((state.layers.clone(), false));
    !state.layers.is_empty()
  }

  /// Puts the scene back as the gesture found it unless `commit`.
  pub(super) fn end_resize(&self, commit: bool) {
    let Ok(mut state) = self.state() else {
      return;
    };
    if let Some((layers, _)) = state.resize.take() {
      if !commit && !layers.is_empty() {
        state.layers = layers;
      }
    }
  }

  /// Resizes every layer's canvas together, the screenshot workspace's Frame
  /// gesture; `selected` is the layer an auto-fit move carries, if any.
  pub(super) fn update_resize(&self, selected: Option<u32>, change: FrameChange) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    if !change.valid() || !state.has_snapshot() {
      return false;
    }
    let mut layers = state.gesture_base();
    for (index, layer) in layers.iter_mut().enumerate() {
      let (next, origin, moved) = resized(layer, change);
      let moved = if selected == Some(index as u32) {
        moved
      } else {
        (0.0, 0.0)
      };
      layer.shift_content(moved.0 - origin.0, moved.1 - origin.1);
      scale_background_radius(layer, next);
    }
    state.applied(layers);
    true
  }

  /// An Option-held move that grows one recording pane's canvas round its
  /// moved clip. A camera baked into a lone screen pane moves on its own,
  /// while the screen and its cursor only follow the canvas origin.
  pub(super) fn update_recording_auto_fit_move(&self, pane: u32, change: FrameChange) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    if !change.valid() || !state.has_snapshot() {
      return false;
    }
    let mut layers = state.gesture_base();
    let baked_camera = pane == 1 && layers.len() == 1;
    let mut found = false;
    for layer in layers
      .iter_mut()
      .filter(|layer| layer.pane_index == pane || baked_camera)
    {
      found = true;
      let (next, origin, moved) = resized(layer, change);
      layer.geometry.size = next;
      if baked_camera {
        layer.shift_content(-origin.0, -origin.1);
        if let Some(camera) = layer.camera.as_mut() {
          camera.geometry.frame_x += (moved.0 - origin.0).round() as i32;
          camera.geometry.frame_y += (moved.1 - origin.1).round() as i32;
        }
      } else {
        layer.shift_content(moved.0 - origin.0, moved.1 - origin.1);
      }
    }
    if found {
      state.applied(layers);
    }
    found
  }

  /// Resizes only `pane`'s canvas, leaving every other pane as it is. A
  /// camera baked into it keeps its place on the canvas while the origin
  /// moves under it.
  pub(super) fn update_selected_resize(&self, pane: u32, change: FrameChange) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    if !change.valid() {
      return false;
    }
    let mut layers = state.gesture_base();
    let mut found = false;
    for layer in layers.iter_mut().filter(|layer| layer.pane_index == pane) {
      found = true;
      let (next, origin, _) = resized(layer, change);
      layer.shift_content(-origin.0, -origin.1);
      if let Some(camera) = layer
        .camera
        .as_mut()
        .filter(|camera| camera.geometry.frame_width > 0)
      {
        camera.geometry.frame_x -= origin.0.round() as i32;
        camera.geometry.frame_y -= origin.1.round() as i32;
      }
      scale_background_radius(layer, next);
    }
    if found {
      state.applied(layers);
    }
    found
  }

  /// The live corner radius of `pane`, as the export derives it: `frame`
  /// rounds the canvas, a share of its shorter side, and otherwise the clip,
  /// a share of the crop's shorter side.
  pub(super) fn update_selected_radius(&self, pane: u32, percent: f64, frame: bool) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    if !percent.is_finite() {
      return false;
    }
    let percent = percent.clamp(0.0, 50.0);
    let mut found = false;
    for layer in state
      .layers
      .iter_mut()
      .filter(|layer| layer.pane_index == pane)
    {
      found = true;
      let geometry = &mut layer.geometry;
      if frame {
        let shortest = f64::from(geometry.size.0.min(geometry.size.1));
        geometry.background_radius = (shortest * percent / 100.0).round() as f32;
      } else {
        let placement = geometry.placement;
        let shortest = f64::from(placement.crop_width.min(placement.crop_height));
        geometry.radius = (shortest * percent / 100.0).round() as f32;
      }
    }
    found
  }

  /// Places `pane`'s shortcut strip at a normalised centre and scales it by
  /// `scale` from where the gesture found it.
  pub(super) fn update_keyboard(&self, pane: u32, center: (f64, f64), scale: f64) -> bool {
    let Ok(mut state) = self.state() else {
      return false;
    };
    if !center.0.is_finite() || !center.1.is_finite() || !scale.is_finite() || scale <= 0.0 {
      return false;
    }
    let scale = scale as f32;
    let mut layers = state.gesture_base();
    let mut found = false;
    for keyboard in layers
      .iter_mut()
      .filter(|layer| layer.pane_index == pane)
      .filter_map(|layer| layer.keyboard.as_mut())
      .filter(|keyboard| keyboard.key_count > 0)
    {
      found = true;
      keyboard.center_x = center.0 as f32;
      keyboard.center_y = center.1 as f32;
      keyboard.requested_scale *= scale;
      keyboard.scale *= scale;
      let count = (keyboard.key_count as usize).min(keyboard.keys.len());
      for key in &mut keyboard.keys[..count] {
        key.scale *= scale;
      }
    }
    if found {
      state.applied(layers);
    }
    found
  }

  /// Where `pane`'s shortcut strip shows, x, y, width and height as shares
  /// of its canvas.
  pub(super) fn keyboard_bounds(&self, pane: u32) -> Option<[f64; 4]> {
    let state = self.state().ok()?;
    let layer = state.layers.iter().find(|layer| {
      layer.pane_index == pane
        && layer
          .keyboard
          .is_some_and(|keyboard| keyboard.key_count > 0)
        && layer.geometry.size.0 > 0
        && layer.geometry.size.1 > 0
    })?;
    state
      .compositor
      .keyboard_visible_bounds(layer.keyboard.as_ref()?, layer.geometry.size)
      .ok()
      .flatten()
  }
}
