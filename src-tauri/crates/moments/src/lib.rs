// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moments: points in a recording the user flags while it runs, each of a
//! kind they named in Settings, so whoever edits it later can find the parts
//! worth keeping.
//!
//! A press is written straight to the recording's moments sidecar
//! (`recorder`, `format`), which carries each kind's name and colour as they
//! were, so later changes in Settings never rewrite a past recording.
//! Held long enough, a press also records a voice note (`voice`,
//! `note_audio`), kept out of the recording's own microphone by the capture
//! crate's `note_gate`. Once a recording stops, each note's transcript is
//! added to the same file (`transcripts`).

pub mod format;
pub mod note_audio;
pub mod recorder;
pub mod settings;
#[cfg(test)]
mod tests;
pub mod transcripts;
pub mod voice;

pub use format::{for_project, note_path, read};
pub use recorder::MomentRecorder;
pub use transcripts::add_transcript;
