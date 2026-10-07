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
//! Held long enough, a press also records a voice note (`voice`,
//! `note_audio`), kept out of the recording's own microphone by
//! `recording::note_gate`.

mod format;
pub(crate) mod note_audio;
mod recorder;
pub(crate) mod settings;
pub(crate) mod shortcuts;
#[cfg(test)]
mod tests;
mod voice;

pub(crate) use format::{note_path, read};
pub(crate) use recorder::MomentRecorder;
