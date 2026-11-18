// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Notes from a call, taken without keeping its audio. Dictation's helper
//! writes each stretch of the microphone and of the computer's audio once a
//! pause ends it (`screenwide_speech::runner::notes`); this crate makes the
//! notes of those lines: the call heard again through the speakers taken out
//! of the microphone's lines (`echo`), the voices in the computer's audio
//! told apart once the call ends (`speakers`), and the whole written as
//! Markdown (`document`), saved as it grows. A few seconds of each voice are
//! kept beside the notes until its speaker is named (`clips`).

pub mod clips;
mod document;
mod echo;
pub mod sensitive;
mod session;
mod speakers;

pub use document::{retitle, Document};
pub use screenwide_speech::runner::notes::{NoteLine, NoteSource, NotesModels, SpeakerTurn};
pub use session::{Failure, Session, Setup};
pub use speakers::Who;
