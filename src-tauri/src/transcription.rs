// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Speech to text, run on this computer, and the other speech models the
//! app downloads. The app keeps the models (`models`, `download`, from the
//! `screenwide_speech` catalogue) and the language to transcribe in
//! (`language`); the models run in the separate `screenwide-transcriber`
//! program (`screenwide_speech::runner`), so a GPU
//! runtime the computer lacks stops only that program and its memory is all
//! given back when it exits. Voice notes are queued for it in `notes`; the
//! microphone's Studio sound uses it from the editor.
//!
//! Each use has a model of its own, so nobody has to pick one; a
//! transcription whose model is missing borrows any other transcription
//! model that is downloaded.

pub(crate) mod commands;
mod download;
pub(crate) mod language;
pub(crate) mod models;
pub(crate) mod notes;
#[cfg(test)]
mod tests;
