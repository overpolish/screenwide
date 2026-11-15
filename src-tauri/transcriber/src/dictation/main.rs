// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation's helper: reads requests from standard input until it closes,
//! running each dictation asked for and answering on standard output, as the
//! transcriber does. The model stays loaded between dictations, so the app
//! keeps the helper while dictation may be used again soon.

mod stream;

use std::io::{BufRead, Write};

use screenwide_transcriber::{Job, Reply, Request};

fn main() {
  transcribe_cpp::disable_logging();
  let mut dictation = stream::Dictation::default();
  let stdin = std::io::stdin();
  let mut stdout = std::io::stdout().lock();
  let mut lines = stdin.lock().lines();
  while let Some(line) = lines.next() {
    let Ok(line) = line else { break };
    if line.trim().is_empty() {
      continue;
    }
    // Without an id there is no request to answer; the app sees the helper
    // go and gives up on the dictation.
    let Ok(Request { id, job }) = serde_json::from_str::<Request>(&line) else {
      eprintln!("Could not read a request");
      break;
    };
    let listening = match job {
      Job::BeginDictation { model } => dictation.run(id, &model, &mut lines, &mut |reply| {
        send(&mut stdout, reply)
      }),
      _ => send(
        &mut stdout,
        &Reply::Failed {
          id,
          message: "This helper only dictates".to_owned(),
        },
      ),
    };
    if !listening {
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
