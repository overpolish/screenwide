// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotation clips put in the order that draws them as they were drawn
//! before a document's order was its stacking.

use crate::editor::annotations::timing::RecordingAnnotationClip;
use crate::editor::annotations::AnnotationKind;

/// Puts `clips` in the order the compositor draws them in: every highlight,
/// then every spotlight, then the rest, each group in the order it was kept.
/// A document written before its order was its stacking drew highlights under
/// the spotlights' shade and that shade under everything else, whatever the
/// order, and so does a recording's live overlay; this keeps either looking
/// as it did once it is drawn in document order.
pub(super) fn stack_by_kind(clips: &mut [RecordingAnnotationClip]) {
  clips.sort_by_key(|clip| match clip.annotation.shape.kind() {
    AnnotationKind::Highlight => 0,
    AnnotationKind::Spotlight => 1,
    _ => 2,
  });
}
