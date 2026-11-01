// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a picture dropped on the workspace lands. The platform receives the
//! drop; which picture it lands on is read as a press with the image tool
//! would read it.

use super::annotation::{drawing_pictures, take_drawing_layer};
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
      // On each picture standing upright, as a press with the image tool
      // reads it.
      let display = (client.0 / scale, client.1 / scale);
      let pictures = drawing_pictures(&state, display);
      let at = drawing_layer(pictures.iter().copied()).and_then(|layer| {
        let &(_, [x, y, width, height], point) =
          pictures.iter().find(|(picture, ..)| *picture == layer)?;
        (width > 0.0 && height > 0.0)
          .then(|| (layer, (point.0 - x) / width, (point.1 - y) / height))
      });
      (at, take_drawing_layer(inner, &mut state, display))
    };
    if let Some(layer) = taken {
      emit_selection(inner, Some(layer));
    }
    at
  }
}
