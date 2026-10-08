// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Speech to text, run on this computer. The app keeps the models
//! (`catalogue`, `models`, `download`) and the language to transcribe in
//! (`language`); the transcribing itself happens in the separate
//! `screenwide-transcriber` program (`runner`), so a GPU runtime the
//! computer lacks stops only that program and its memory is all given back
//! when it exits. Voice notes are queued for it in `notes`.
//!
//! Each use of transcription has a model of its own, so nobody has to pick
//! one; a use whose model is missing borrows any other that is downloaded.

mod catalogue;
pub(crate) mod commands;
mod download;
pub(crate) mod language;
mod models;
pub(crate) mod notes;
pub(crate) mod runner;
#[cfg(test)]
mod tests;
