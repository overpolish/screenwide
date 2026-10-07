// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moments: points in a recording the user flags while it runs, each of a
//! kind they named in Settings, so whoever edits it later can find the parts
//! worth keeping.
//!
//! A kind's shortcut is claimed only while a recording runs (`shortcuts`).
//! A press is written straight to the recording's moments sidecar
//! (`recorder`, `format`), which carries each kind's name and colour as they
//! were, so later changes in Settings never rewrite a past recording.

mod format;
mod recorder;
pub(crate) mod settings;
pub(crate) mod shortcuts;
#[cfg(test)]
mod tests;

pub(crate) use format::read;
pub(crate) use recorder::MomentRecorder;
