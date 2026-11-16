// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Takes notes from two recordings of a call, the microphone and the
//! computer's audio, as notes would be taken from the call itself: each fed a
//! sixteenth of a second at a time, in turn. Prints the notes and how long
//! finishing took. Needs `pnpm transcriber:prepare` and FFmpeg on the PATH:
//!
//! ```sh
//! cargo run -p screenwide-notes --example notes_from_files -- \
//!   microphone.wav system.wav notes.md \
//!   parakeet-ultra-Q8_0.gguf silero-vad.bin Nemotron-3-Diarization.q8_0.gguf \
//!   3dspeaker_speech_campplus_sv_en_voxceleb_16k.onnx
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use screenwide_notes::{Document, NoteSource};
use screenwide_speech::runner::notes::NotesModels;
use screenwide_speech::runner::Transcriber;

/// A sixteenth of a second at the protocol's rate.
const FEED: usize = 1_600;

fn main() -> Result<(), String> {
  let arguments: Vec<String> = std::env::args().skip(1).collect();
  let [microphone, system, notes, model, vad, speakers, voices] = arguments.as_slice() else {
    return Err("Takes microphone system notes model vad speakers voices".to_owned());
  };
  let microphone = decoded(Path::new(microphone));
  let system = decoded(Path::new(system));
  let mut helper = Transcriber::start_dictation()?;
  helper.begin_notes(
    NotesModels {
      model: PathBuf::from(model),
      vad: PathBuf::from(vad),
      speakers: PathBuf::from(speakers),
      voices: PathBuf::from(voices),
    },
    Some("en".to_owned()),
    Vec::new(),
  )?;
  let mut document = Document::new("Notes".to_owned());
  let chunks = microphone.len().max(system.len()).div_ceil(FEED);
  for chunk in 0..chunks {
    let at_ms = (chunk * FEED * 1_000 / 16_000) as u64;
    for (source, audio) in [
      (NoteSource::Microphone, &microphone),
      (NoteSource::System, &system),
    ] {
      let start = (chunk * FEED).min(audio.len());
      let end = (start + FEED).min(audio.len());
      let lines = helper.feed_notes(source, at_ms, audio[start..end].to_vec())?;
      document.add(lines);
    }
  }
  let started = Instant::now();
  let finished = helper.finish_notes()?;
  document.finish(finished.lines, finished.turns);
  let notes = Path::new(notes);
  document.save(notes).map_err(|error| error.to_string())?;
  let root = notes.parent().unwrap_or(Path::new("."));
  screenwide_notes::clips::save(root, &finished.voices, &document.voices())
    .map_err(|error| error.to_string())?;
  println!("{}", document.markdown());
  println!("Finished in {} ms", started.elapsed().as_millis());
  Ok(())
}

/// The file at `path` as 16 kHz mono 16-bit samples.
fn decoded(path: &Path) -> Vec<i16> {
  let Ok(output) = Command::new("ffmpeg")
    .args(["-loglevel", "error", "-i"])
    .arg(path)
    .args(["-f", "s16le", "-ar", "16000", "-ac", "1", "-"])
    .output()
  else {
    return Vec::new();
  };
  let (samples, _) = output.stdout.as_chunks::<2>();
  samples
    .iter()
    .map(|&pair| i16::from_le_bytes(pair))
    .collect()
}
