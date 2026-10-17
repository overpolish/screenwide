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
    let mut selection_stale = false;
    for pane in state.panes.iter_mut().flatten() {
      selection_stale |= std::mem::take(&mut pane.selection_stale);
    }
    if state.workspace.pending {
      draw_workspace(inner, &mut state);
    }
    if inner.selection_pending.swap(false, Ordering::AcqRel) || selection_stale {
      draw_selection(inner, &state);
    }
    // Only a change to the visual tree needs committing, such as a viewport
    // `finish_layout` left to this flush. Frames alone reach the screen
    // through their swap chains, and committing for them anyway would cost
    // every arrow-drag sample a display tick of wait while the grips, which
    // never commit, run ahead of the arrow.
    if inner.commit_pending.swap(false, Ordering::AcqRel)
      && unsafe { inner.gpu.composition.Commit() }.is_ok()
    {
      // As in `finish_layout`: an unawaited commit backlog lets rapid
      // drags visibly desynchronise the workspace from the DOM controls.
      let _ = unsafe { inner.gpu.composition.WaitForCommitCompletion() };
    }
  }
}

impl RecordingPreviewSurface {
  /// Opens a present batch: the workspace draws once when the guard drops,
  /// with every layout and present made meanwhile. Dropping the guard draws
  /// even when nothing was presented, so a deferred layout never strands the
  /// picture.
  pub(crate) fn present_batch(&self) -> PresentBatch<'_> {
    self.inner.batch_depth.fetch_add(1, Ordering::AcqRel);
    PresentBatch { surface: self }
  }
}
