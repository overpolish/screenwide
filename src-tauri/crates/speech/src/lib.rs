// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The microphone's speech tools. Remove silences (`silences`) cuts the long
//! pauses from one listen through the microphone track for speech
//! (`analysis`), kept beside the project so it runs once per recording.
//! Studio sound (`studio`) rebuilds the voice as a studio recording; where
//! it is off, Reduce noise (`noise`) cleans the track with a speech enhancer
//! and Vocal cleanup (`voice`) polishes the voice. `heard` picks which of
//! their files the track is heard from. Auto volume (`auto_volume`) levels
//! whatever is heard to a steady loudness, and makes the system audio make
//! way for the voice while it speaks (`duck`), the singing in it out of the
//! way entirely with Studio ducking (`studio_ducking`).
//! `catalogue` lists the models they run on, and `runner` starts the
//! speech-to-text helper and decodes audio for it.

pub mod analysis;
pub mod auto_volume;
pub mod catalogue;
pub mod choice_file;
pub mod duck;
mod framer;
pub mod heard;
pub mod noise;
mod pause_gate;
pub mod runner;
pub mod silences;
pub mod studio;
#[cfg(test)]
mod tests;
pub mod transcript;
pub mod voice;
