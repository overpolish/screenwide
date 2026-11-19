// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The copies a carry takes with the duplicate modifier held, on a
//! recording: each gets a clip just above its original's, timed as it is,
//! and React is told whose copy each is, so the clip it makes is timed the
//! same rather than placed at the playhead as a fresh annotation's is.

use super::gesture::Commit;
use super::*;
use screenwide_annotations::copy::{copy_clip_in_place, Copied};

/// What a carry with the duplicate modifier took: the copies, and the clips
/// as they were before, which a cancel puts back.
pub(super) struct Copies {
  pub(super) copied: Vec<Copied>,
  pub(super) restore: Vec<RecordingAnnotationClip>,
}

impl Copies {
  /// The originals' ids, chosen again when the carry is cancelled.
  pub(super) fn originals(&self) -> Vec<String> {
    self
      .copied
      .iter()
      .map(|copied| copied.original.clone())
      .collect()
  }

  /// The copies' ids, which the carry moves and leaves chosen.
  pub(super) fn copies(&self) -> Vec<String> {
    self
      .copied
      .iter()
      .map(|copied| copied.copy.clone())
      .collect()
  }
}

/// A copy a carry took, and the annotation it copies.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CommitCopy {
  pub(super) annotation_id: String,
  pub(super) original_id: String,
}

/// `before` with a clip for each of `copied`, just above its original's,
/// with the copy as `shown` holds it; and what a cancel puts back. None
/// where nothing was copied.
pub(super) fn with_copies(
  before: Vec<RecordingAnnotationClip>,
  copied: Option<Copied>,
  shown: &[Annotation],
) -> (Vec<RecordingAnnotationClip>, Option<Copies>) {
  let Some(copied) = copied else {
    return (before, None);
  };
  let mut clips = before.clone();
  if let Some(copy) = shown.iter().find(|annotation| annotation.id == copied.copy) {
    copy_clip_in_place(&mut clips, &copied, copy);
  }
  let copies = Copies {
    copied: vec![copied],
    restore: before,
  };
  (clips, Some(copies))
}

impl Commit {
  /// The commit, telling React whose copy each copy is.
  pub(super) fn with_copies(self, copies: Option<&Copies>) -> Self {
    let copies = copies
      .map(|copies| {
        copies
          .copied
          .iter()
          .map(|copied| CommitCopy {
            annotation_id: copied.copy.clone(),
            original_id: copied.original.clone(),
          })
          .collect()
      })
      .unwrap_or_default();
    Self { copies, ..self }
  }
}
