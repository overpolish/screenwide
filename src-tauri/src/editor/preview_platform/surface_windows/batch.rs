// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Drop for PresentBatch<'_> {
  fn drop(&mut self) {
    let inner = &self.surface.inner;
    if inner.batch_depth.fetch_sub(1, Ordering::AcqRel) != 1 {
      return;
    }
    let Ok(mut state) = inner.state.lock() else {
      return;
    };
    for pane in state.panes.iter_mut().flatten() {
      if pane.pending_present {
        pane.pending_present = false;
        let _ = unsafe { pane.swap_chain.Present(0, DXGI_PRESENT(0)) }.ok();
      }
    }
    let mut selection_stale = false;
    for pane in state.panes.iter_mut().flatten() {
      if pane.pending_geometry {
        pane.pending_geometry = false;
        let _ = pane.update_geometry();
        // Selection geometry is derived from the pane's final canvas rect.
        // A Frame undo can move/resize that rect without resizing the D3D
        // buffer, so buffer staleness alone is not sufficient.
        selection_stale = true;
      }
      selection_stale |= std::mem::take(&mut pane.selection_stale);
    }
    if inner.selection_pending.swap(false, Ordering::AcqRel) || selection_stale {
      draw_selection(inner, &state);
    }
    // Unconditional: `finish_layout` leaves its hides to this commit whenever
    // the batch was already open.
    if unsafe { inner.gpu.composition.Commit() }.is_ok() {
      // As in `finish_layout`: an unawaited commit backlog lets rapid drags
      // visibly desynchronise the panes from the DOM controls above them.
      let _ = unsafe { inner.gpu.composition.WaitForCommitCompletion() };
    }
  }
}

impl RecordingPreviewSurface {
  /// Opens a present batch: frames drawn until the guard drops are parked on
  /// their panes and published by one flush together with every geometry
  /// deferred by `layout`. Dropping the guard flushes even when nothing was
  /// presented, so a deferred layout never strands the panes.
  pub(crate) fn present_batch(&self) -> PresentBatch<'_> {
    self.inner.batch_depth.fetch_add(1, Ordering::AcqRel);
    PresentBatch { surface: self }
  }
}
