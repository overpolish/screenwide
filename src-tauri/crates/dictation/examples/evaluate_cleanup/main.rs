// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scores cleanup models on the recorded dictations in `evaluation/`: each
//! is heard as dictation hears it, its spoken commands carried out, then
//! corrected by every model, and the answers scored for corrections applied,
//! clean text left as it was, and words lost, with and without the check
//! that puts back the text as heard (`score`). Writes
//! `evaluation/results.md`; `evaluation/README.md` says how to run it.

mod score;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use screenwide_dictation::correct::{as_said, kept, Prompt, GEMMA_4, S1_MINI, SPEAKOFLOW_MINI};
use screenwide_dictation::{phrases, shape};
use screenwide_speech::runner::Transcriber;

use self::score::{report, Answer, Case, Scored};

/// A sixteenth of a second at the protocol's rate, as the microphone feeds
/// dictation.
const FEED: usize = 1_600;

fn main() -> Result<(), String> {
  let mut recognizer = None;
  let mut models = Vec::new();
  let mut arguments = std::env::args().skip(1);
  while let Some(argument) = arguments.next() {
    let value = arguments
      .next()
      .ok_or(format!("{argument} needs a value"))?;
    match argument.as_str() {
      "--recognizer" => recognizer = Some(PathBuf::from(value)),
      "--model" => {
        let (name, path) = value.split_once('=').ok_or("--model takes name=path")?;
        models.push((name.to_owned(), prompt(name)?, PathBuf::from(path)));
      }
      _ => return Err(format!("Unknown argument {argument}")),
    }
  }
  let recognizer = recognizer.ok_or("--recognizer is needed")?;
  let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("evaluation");
  let cases = heard(&here, &recognizer)?;
  let mut scored = vec![Scored {
    name: "none (as heard)".to_owned(),
    answers: cases
      .iter()
      .map(|case| Answer {
        text: case.heard.clone(),
        kept: true,
        milliseconds: 0,
      })
      .collect(),
  }];
  for (name, prompt, path) in models {
    eprintln!("Correcting with {name}");
    scored.push(Scored {
      answers: corrected(&cases, prompt, &path)?,
      name,
    });
  }
  let report = report(&cases, &scored);
  std::fs::write(here.join("results.md"), report).map_err(|error| error.to_string())
}

/// The prompt for the model `name`d, which may add its quantisation
/// (`speakoflow-mini-q8`).
fn prompt(name: &str) -> Result<Prompt, String> {
  [
    ("speakoflow-mini", SPEAKOFLOW_MINI),
    ("s1-mini", S1_MINI),
    ("gemma-4-e2b", GEMMA_4),
  ]
  .into_iter()
  .find(|(model, _)| name.starts_with(model))
  .map(|(_, prompt)| prompt)
  .ok_or_else(|| format!("No prompt for {name}"))
}

/// Every case as dictation hears it with the model at `recognizer`.
fn heard(here: &Path, recognizer: &Path) -> Result<Vec<Case>, String> {
  let source =
    std::fs::read_to_string(here.join("cases.json")).map_err(|error| error.to_string())?;
  let json: serde_json::Value = serde_json::from_str(&source).map_err(|error| error.to_string())?;
  let field =
    |case: &serde_json::Value, name: &str| case[name].as_str().unwrap_or_default().to_owned();
  let mut dictation = Transcriber::start_dictation()?;
  let mut cases = Vec::new();
  for case in json["cases"].as_array().ok_or("cases.json has no cases")? {
    let id = field(case, "id");
    let samples = decoded(&here.join("audio").join(format!("{id}.m4a")))?;
    dictation.begin_dictation(recognizer.to_owned(), vec!["Screenwide".to_owned()])?;
    for chunk in samples.chunks(FEED) {
      dictation.feed_dictation(chunk.to_vec())?;
    }
    let text = dictation.finish_dictation()?;
    let heard = shape(&text, phrases(Some("en"))).text();
    eprintln!("{id}: {heard:?}");
    cases.push(Case {
      category: field(case, "category"),
      expect: field(case, "expect"),
      heard,
      id,
    });
  }
  Ok(cases)
}

/// The file at `path` as 16 kHz mono 16-bit samples.
fn decoded(path: &Path) -> Result<Vec<i16>, String> {
  let output = Command::new("ffmpeg")
    .args(["-loglevel", "error", "-i"])
    .arg(path)
    .args(["-f", "s16le", "-ar", "16000", "-ac", "1", "-"])
    .output()
    .map_err(|error| format!("Could not run FFmpeg: {error}"))?;
  let (samples, _) = output.stdout.as_chunks::<2>();
  Ok(
    samples
      .iter()
      .map(|&pair| i16::from_le_bytes(pair))
      .collect(),
  )
}

/// Every case corrected by the model at `path`, once it is loaded.
fn corrected(cases: &[Case], prompt: Prompt, path: &Path) -> Result<Vec<Answer>, String> {
  let mut cleanup = Transcriber::start_cleanup()?;
  let mut correct = |text: &str| {
    cleanup.correct_text(
      path.to_owned(),
      prompt.instructions.to_owned(),
      prompt.message(text),
    )
  };
  correct("Loading the model.")?;
  cases
    .iter()
    .map(|case| {
      let started = Instant::now();
      let text = as_said(&case.heard, &correct(&case.heard)?);
      Ok(Answer {
        kept: kept(&case.heard, &text),
        milliseconds: started.elapsed().as_millis(),
        text,
      })
    })
    .collect()
}
