// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reads requests from standard input until it closes, answering each on
//! standard output. Engine logs are dropped: standard error is kept for this
//! program's own failures, which the app shows.

mod engine;

use std::io::{BufRead, Write};

use screenwide_transcriber::{Reply, Request};

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
      Ok(request) => match engine.transcribe(&request) {
        Ok(transcript) => Reply::Done {
          id: request.id,
          transcript,
        },
        Err(message) => Reply::Failed {
          id: request.id,
          message,
        },
      },
      // Without an id there is no request to answer; the app sees the
      // helper go and gives up on whatever it was waiting for.
      Err(error) => {
        eprintln!("Unreadable request: {error}");
        std::process::exit(2);
      }
    };
    let Ok(json) = serde_json::to_string(&reply) else {
      std::process::exit(3);
    };
    if writeln!(stdout, "{json}")
      .and_then(|()| stdout.flush())
      .is_err()
    {
      break;
    }
  }
}
