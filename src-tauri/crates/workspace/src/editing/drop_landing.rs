// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a picture dropped on the workspace lands. The platform receives the
//! drop; which picture it lands on is read as a press with the image tool
//! would read it.

use super::annotation::{layer_image_rect, take_drawing_layer};
use super::*;
use screenwide_annotations::gesture::drawing_layer;

impl WorkspaceEditor {
  /// Whether a drop is taken now: the editor showing and not covered by the
  /// frontend's own chrome.
  pub fn takes_drops(&self) -> bool {
    !self.inner.host.is_suspended()
      && self
        .inner
        .state
        .lock()
        .is_ok_and(|state| state.editor_active)
  }

  /// Where a drop at `client`, in the drawable's pixels, lands: the layer a
  /// press there would draw on, by the rule [`drawing_layer`] keeps, and the
  /// point in its image-normalised space - beyond 0 to 1 for a drop on the
  /// frame. The layer is taken in hand, and reported, where that changes the
  /// choice, as a press with the image tool there would.
  pub fn land_drop(&self, client: (f64, f64)) -> Option<(u32, f64, f64)> {
    let inner = &self.inner;
    let (at, taken) = {
      let mut state = inner.state.lock().ok()?;
      let scale = state.scale.max(0.1);
      // On the layer standing upright, as a press with the image tool reads it.
      let point = turn::selected_layer_point(&state, (client.0 / scale, client.1 / scale));
      let pictures: Vec<_> = state
        .selection_targets
        .iter()
        .filter_map(|target| {
          let rect = layer_image_rect(&state, target.layer_id as i32)?;
          Some((target.layer_id, [rect.x, rect.y, rect.width, rect.height]))
        })
        .collect();
      let at = drawing_layer(pictures.into_iter(), point).and_then(|layer| {
        let image = layer_image_rect(&state, layer as i32)?;
        (image.width > 0.0 && image.height > 0.0).then(|| {
          (
            layer,
            (point.0 - image.x) / image.width,
            (point.1 - image.y) / image.height,
          )
        })
      });
      (at, take_drawing_layer(inner, &mut state, point))
    };
    if let Some(layer) = taken {
      emit_selection(inner, Some(layer));
    }
    at
  }
}
