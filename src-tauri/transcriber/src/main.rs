// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reads requests from standard input until it closes, answering each on
//! standard output. Engine logs are dropped: standard error is kept for this
//! program's own failures, which the app shows.

mod clean;
mod engine;

use std::io::{BufRead, Write};

use screenwide_transcriber::{Job, Reply, Request};

fn main() {
  whisper_rs::install_logging_hooks();
  let mut engine = engine::Engine::default();
  let stdin = std::io::stdin();
  let mut stdout = std::io::stdout().lock();
  for line in stdin.lock().lines() {
    let Ok(line) = line else { break };
    if line.trim().is_empty() {
      continue;
    }
    let reply = match serde_json::from_str::<Request>(&line) {
      Ok(Request { id, job }) => {
        let mut progress = |fraction: f32| {
          send(&mut stdout, &Reply::Progress { id, fraction });
        };
        let result = match &job {
          Job::Transcribe {
            model,
            model_id,
            audio,
            language,
          } => engine
            .transcribe(model, model_id, audio, language.as_deref())
            .map(|transcript| Reply::Done { id, transcript }),
          Job::DetectSpeech { model, audio } => engine
            .detect_speech(model, audio, &mut progress)
            .map(|speech| Reply::Speech { id, speech }),
          Job::CleanSpeech { audio, output } => clean::clean_speech(audio, output, &mut progress)
            .map(|gain| Reply::Cleaned { id, gain }),
        };
        result.unwrap_or_else(|message| Reply::Failed { id, message })
      }
      // Without an id there is no request to answer; the app sees the
      // helper go and gives up on whatever it was waiting for.
      Err(error) => {
        eprintln!("Unreadable request: {error}");
        std::process::exit(2);
      }
    };
    if !send(&mut stdout, &reply) {
      break;
    }
  }
}

/// Writes `reply` as one line. False once the app has stopped listening.
fn send(stdout: &mut std::io::StdoutLock<'_>, reply: &Reply) -> bool {
  let Ok(json) = serde_json::to_string(reply) else {
    std::process::exit(3);
  };
  writeln!(stdout, "{json}")
    .and_then(|()| stdout.flush())
    .is_ok()
}
