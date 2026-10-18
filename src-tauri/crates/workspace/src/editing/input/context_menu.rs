// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The annotation or layer under a right press, handed to the web layer so it
/// can open the app's own menu there. An annotation comes before the layer it
/// is drawn on, as it does for a left press, and is chosen the same way so the
/// menu acts on what it was opened on.
pub(super) fn open(inner: &std::sync::Arc<SurfaceInner>, point: (f64, f64)) {
  if inner.host.is_suspended() {
    return;
  }
  {
    let Ok(state) = inner.state.lock() else {
      return;
    };
    if !state.editor_active || state.gesture.is_some() {
      return;
    }
  }
  // A press on the caption being typed into opens the word menu for the
  // word under it, named by the byte of the typed text it landed at, and
  // the typing goes on.
  if let Some(offset) = annotation::caption_offset_at(inner, turn::layer_point(inner, point)) {
    let Ok(state) = inner.state.lock() else {
      return;
    };
    let position = content_point(&state, point);
    drop(state);
    report(
      inner,
      crate::CAPTIONS_LAYER_ID,
      Some(u32::try_from(offset).unwrap_or(u32::MAX)),
      position,
    );
    return;
  }
  if let Some(annotation) = annotation::choose_at(inner, point) {
    let Ok(state) = inner.state.lock() else {
      return;
    };
    let layer = state.selection.map_or(0, |selection| selection.layer_id);
    let position = content_point(&state, point);
    drop(state);
    report(inner, layer, Some(annotation as u32), position);
    return;
  }
  let Ok(mut state) = inner.state.lock() else {
    return;
  };
  // A press on a word of the caption opens the captions' word menu for it,
  // with the select tool or the Captions tool in hand. The caption is drawn
  // over every layer, so it comes before them.
  if let Some(target) = annotation::caption_target_at(inner, &state, point) {
    let Some(word) = caption_word_at(inner, &state, target, point) else {
      return;
    };
    state.annotation.caption = Some(target);
    let position = content_point(&state, point);
    drop(state);
    report(inner, target.layer_id, Some(word), position);
    return;
  }
  // Layers are only picked while the select tool is in hand; an annotation
  // tool picks annotations, above, and nothing else.
  if !state.selection_hit_testing {
    return;
  }
  let Some((target, _)) = shared_selection_hit(inner, &state, point) else {
    return;
  };
  // Frames and the overlays drawn over the picture have no layer-order menu.
  if target.layer_id == FRAME_LAYER_ID || crate::is_overlay_layer(target.layer_id) {
    return;
  }
  let changed = state.selection.is_none_or(|current| {
    current.pane_index != target.pane_index || current.layer_id != target.layer_id
  });
  state.last_pointer = point;
  state.selection = Some(target);
  clear_selection_snap_guides(&mut state);
  draw_selection(inner, &state);
  let position = content_point(&state, point);
  drop(state);
  if changed {
    emit_selection(inner, Some(target.layer_id));
  }
  report(inner, target.layer_id, None, position);
}

/// The word of the caption showing under `point`, on the caption layer
/// `target`, counted across its lines: the point is carried from the box the
/// caption's words are shown in back into its canvas, where they are drawn.
fn caption_word_at(
  inner: &SurfaceInner,
  state: &SurfaceState,
  target: PreviewSelection,
  point: (f64, f64),
) -> Option<u32> {
  let pane = state.panes.get(target.pane_index as usize)?.as_ref()?;
  let settings = pane.settings.as_ref()?;
  let (track, moment) = pane.captions.as_ref()?;
  let canvas = (settings.width, settings.height);
  let bounds =
    screenwide_compositor::annotation_gpu::caption_visible_bounds(track, *moment, canvas)?;
  let shown = overlay_hit::frame(inner, state, target)?;
  if shown.width <= 0.0 || shown.height <= 0.0 {
    return None;
  }
  let across = bounds[0] + (point.0 - shown.x) / shown.width * bounds[2];
  let down = bounds[1] + (point.1 - shown.y) / shown.height * bounds[3];
  let faces = screenwide_compositor::annotation_gpu::CaptionFaces::of(&track.recording.style);
  let mut measure = screenwide_compositor::annotation_gpu::caption_measure(&faces);
  let frame = screenwide_text::captions::CaptionFrame {
    width: settings.width,
    height: settings.height,
  };
  let word = track.word_at(
    frame,
    *moment,
    [across * f64::from(canvas.0), down * f64::from(canvas.1)],
    &mut measure,
  );
  word.and_then(|word| u32::try_from(word).ok())
}

/// Pointer input is local to the inset native viewport. React's menu is
/// positioned in the webview's logical content coordinates.
fn content_point(state: &SurfaceState, point: (f64, f64)) -> (f64, f64) {
  (state.viewport.x + point.0, state.viewport.y + point.1)
}

/// Reports the press. MUST be called with no surface state held.
fn report(inner: &SurfaceInner, layer: u32, annotation: Option<u32>, position: (f64, f64)) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.context_menu.as_mut() {
      callback(layer, annotation, position.0, position.1);
    }
  }
}
