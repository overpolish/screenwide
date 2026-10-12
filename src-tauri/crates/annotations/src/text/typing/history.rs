// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Undo while a box is being typed into. The whole typing session is one step
//! in the editor's own history; this is the finer one the keyboard undoes
//! inside it, the way any text field does.

/// The kind of edit an undo step is still collecting: a run of typing is
/// undone as one, and so is a run of erasing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Run {
  Insert,
  Erase,
}

/// The text and the selection, as an undo step restores them.
#[derive(Clone, Debug)]
pub(super) struct Snapshot {
  pub(super) text: String,
  pub(super) anchor: usize,
  pub(super) caret: usize,
}

#[derive(Clone, Debug, Default)]
pub(super) struct History {
  undo: Vec<Snapshot>,
  redo: Vec<Snapshot>,
  run: Option<Run>,
}

impl History {
  /// Keeps what an edit of `run`'s kind starts from, unless the step being
  /// collected is already of that kind.
  pub(super) fn record(&mut self, run: Run, before: impl FnOnce() -> Snapshot) {
    if self.run != Some(run) {
      self.undo.push(before());
      self.redo.clear();
    }
    self.run = Some(run);
  }

  /// Ends the step being collected: a move or a paste starts the next edit
  /// afresh.
  pub(super) fn settle(&mut self) {
    self.run = None;
  }

  /// Trades `current` for the step before it, or with `back` false for the
  /// step undone last, and keeps `current` for the way back.
  pub(super) fn step(&mut self, back: bool, current: &mut Snapshot) -> bool {
    let (from, to) = if back {
      (&mut self.undo, &mut self.redo)
    } else {
      (&mut self.redo, &mut self.undo)
    };
    let Some(snapshot) = from.pop() else {
      return false;
    };
    to.push(std::mem::replace(current, snapshot));
    self.run = None;
    true
  }
}
