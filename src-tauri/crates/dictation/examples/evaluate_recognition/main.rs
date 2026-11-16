// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scores the ways English dictation can be recognised on the recorded
//! dictations in `evaluation/`: the macOS voices of `cases.json`, and a
//! person's own recordings of `voice-cases.json` where `voice/` holds any.
//! Each recogniser hears every recording as dictation would, a sixteenth of
//! a second at a time, and is scored for words wrong against what was said,
//! with numbers spelled out (`normalize`), and for how long the last of its
//! text takes once the speaker stops. With `--cleanup`, a person's own
//! recordings are corrected too and scored against the text they should
//! become. Writes `evaluation/recognition.md` (`report`).

mod normalize;
mod report;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use screenwide_dictation::correct::{kept, SPEAKOFLOW_MINI};
use screenwide_dictation::{phrases, shape};
use screenwide_speech::runner::Transcriber;

use self::report::{Heard, Recognised};

/// A sixteenth of a second at the protocol's rate, as the microphone feeds
/// dictation.
const FEED: usize = 1_600;

/// A recording, what was said in it, and for a person's own, the text it
/// should become.
pub struct Recording {
  pub id: String,
  pub own: bool,
  samples: Vec<i16>,
  pub said: String,
  pub expect: Option<String>,
}

enum Writes {
  AsSpoken,
  AtPauses,
  /// Streamed, then each stretch written again by the model at this path.
  Rewritten(PathBuf),
}

struct Recogniser {
  name: String,
  model: PathBuf,
  writes: Writes,
}

fn main() -> Result<(), String> {
  let (mut recognisers, mut vad, mut cleanup) = (Vec::new(), None, None);
  let mut arguments = std::env::args().skip(1);
  while let Some(argument) = arguments.next() {
    let value = arguments
      .next()
      .ok_or(format!("{argument} needs a value"))?;
    let (name, path) = value
      .split_once('=')
      .map_or((String::new(), value.as_str()), |(name, path)| {
        (name.to_owned(), path)
      });
    let model = PathBuf::from(path);
    match argument.as_str() {
      "--stream" => recognisers.push(Recogniser {
        name,
        model,
        writes: Writes::AsSpoken,
      }),
      "--at-pauses" => recognisers.push(Recogniser {
        name: format!("{name} at pauses"),
        model,
        writes: Writes::AtPauses,
      }),
      "--rewritten" => {
        let (stream, rewriter) = path
          .split_once(',')
          .ok_or("--rewritten takes name=stream,rewriter")?;
        recognisers.push(Recogniser {
          name: format!("{name}, written again at pauses"),
          model: PathBuf::from(stream),
          writes: Writes::Rewritten(PathBuf::from(rewriter)),
        });
      }
      "--vad" => vad = Some(model),
      "--cleanup" => cleanup = Some(model),
      _ => return Err(format!("Unknown argument {argument}")),
    }
  }
  let needs_vad = recognisers
    .iter()
    .any(|recogniser| !matches!(recogniser.writes, Writes::AsSpoken));
  let vad = match vad {
    Some(vad) => vad,
    None if needs_vad => return Err("Writing at pauses needs --vad".to_owned()),
    None => PathBuf::new(),
  };
  let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("evaluation");
  let recordings = recordings(&here)?;
  let mut results = Vec::new();
  for recogniser in &recognisers {
    eprintln!("Hearing with {}", recogniser.name);
    let mut heard = heard(&recordings, recogniser, &vad)?;
    if let Some(model) = &cleanup {
      corrected(&recordings, &mut heard, model)?;
    }
    results.push(Recognised {
      name: recogniser.name.clone(),
      heard,
    });
  }
  std::fs::write(
    here.join("recognition.md"),
    report::report(&recordings, &results),
  )
  .map_err(|error| error.to_string())
}

/// The macOS voices' recordings, but those of spoken commands, whose words
/// are not all written; then a person's own, where recorded. A recording
/// FFmpeg cannot read, as one cut off part-way, is left out.
fn recordings(here: &Path) -> Result<Vec<Recording>, String> {
  let read = |name: &str| -> Result<serde_json::Value, String> {
    let source = std::fs::read_to_string(here.join(name)).map_err(|error| error.to_string())?;
    serde_json::from_str(&source).map_err(|error| error.to_string())
  };
  let text =
    |case: &serde_json::Value, name: &str| case[name].as_str().unwrap_or_default().to_owned();
  let mut recordings = Vec::new();
  for case in read("cases.json")?["cases"]
    .as_array()
    .into_iter()
    .flatten()
  {
    if case["category"] == "command" {
      continue;
    }
    let id = text(case, "id");
    let said = case["say"]
      .as_array()
      .into_iter()
      .flatten()
      .filter_map(serde_json::Value::as_str)
      .collect::<Vec<_>>()
      .join(" ");
    recordings.push(Recording {
      samples: decoded(&here.join("audio").join(format!("{id}.m4a"))),
      id,
      own: false,
      said,
      expect: None,
    });
  }
  for case in read("voice-cases.json")?["cases"]
    .as_array()
    .into_iter()
    .flatten()
  {
    let id = text(case, "id");
    let samples = decoded(&here.join("voice").join(format!("{id}.m4a")));
    if samples.is_empty() {
      continue;
    }
    recordings.push(Recording {
      samples,
      id,
      own: true,
      said: text(case, "say"),
      expect: Some(text(case, "expect")),
    });
  }
  Ok(recordings)
}

/// Every recording as `recogniser` hears it, its spoken commands carried
/// out.
fn heard(
  recordings: &[Recording],
  recogniser: &Recogniser,
  vad: &Path,
) -> Result<Vec<Heard>, String> {
  let mut dictation = Transcriber::start_dictation()?;
  let words = vec!["Screenwide".to_owned()];
  let english = || Some("en".to_owned());
  recordings
    .iter()
    .map(|recording| {
      let model = recogniser.model.clone();
      match &recogniser.writes {
        Writes::AsSpoken => dictation.begin_dictation(model, words.clone())?,
        Writes::AtPauses => {
          dictation.begin_dictation_at_pauses((model, vad.to_owned()), english(), words.clone())?;
        }
        Writes::Rewritten(rewriter) => dictation.begin_dictation_rewritten(
          (model, rewriter.clone(), vad.to_owned()),
          english(),
          words.clone(),
        )?,
      }
      for chunk in recording.samples.chunks(FEED) {
        dictation.feed_dictation(chunk.to_vec())?;
      }
      let started = Instant::now();
      let text = dictation.finish_dictation()?;
      let finish_ms = started.elapsed().as_millis();
      let text = shape(&text, phrases(Some("en"))).text();
      eprintln!("  {}: {text:?}", recording.id);
      Ok(Heard {
        text,
        finish_ms,
        corrected: None,
      })
    })
    .collect()
}

/// A person's own recordings, as heard, corrected by the cleanup model at
/// `model`, an answer failing the lost-word check left as heard.
fn corrected(recordings: &[Recording], heard: &mut [Heard], model: &Path) -> Result<(), String> {
  let mut cleanup = Transcriber::start_cleanup()?;
  let instructions = SPEAKOFLOW_MINI.instructions.to_owned();
  cleanup.correct_text(model.to_owned(), instructions.clone(), String::new())?;
  for (recording, heard) in recordings.iter().zip(heard) {
    if !recording.own {
      continue;
    }
    let answer = cleanup.correct_text(
      model.to_owned(),
      instructions.clone(),
      SPEAKOFLOW_MINI.message(&heard.text),
    )?;
    heard.corrected = Some(if kept(&heard.text, &answer) {
      answer
    } else {
      heard.text.clone()
    });
  }
  Ok(())
}

/// The file at `path` as 16 kHz mono 16-bit samples; none if FFmpeg cannot
/// read it.
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
