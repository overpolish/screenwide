// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Corrections made while the speaker goes on. At each pause the text not
//! yet settled (`draft`) is corrected on the cleanup helper, in the
//! background, so the correction shows while the speaker is still talking.
//! When dictation stops, only what is still open is left to correct.

use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use screenwide_speech::runner::Transcriber;

use super::draft::Draft;
use super::{as_said, kept, Prompt};

/// The cleanup model, and how it is asked.
pub struct CleanupModel {
  pub path: PathBuf,
  pub prompt: Prompt,
}

#[derive(Default)]
struct State {
  draft: Draft,
  /// Open text waiting to be corrected, and the draft's round it belongs
  /// to.
  wanted: Option<(u64, String)>,
  /// The helper could not start, or the dictation is over.
  stopped: bool,
}

type Shared = Arc<(Mutex<State>, Condvar)>;

fn locked(shared: &Shared) -> MutexGuard<'_, State> {
  shared.0.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Corrects one dictation as it goes on. Dropped, it stops correcting.
pub struct Corrector {
  shared: Shared,
}

impl Corrector {
  /// Starts correcting with `model` on `helper`, or a helper of its own,
  /// loading the model at once so the first pause is not kept waiting.
  /// `corrected` is called whenever a correction comes back, and `keep` is
  /// handed the helper once the dictation is over and it is idle.
  pub fn start(
    model: CleanupModel,
    helper: Option<Transcriber>,
    corrected: impl Fn() + Send + 'static,
    keep: impl FnOnce(Option<Transcriber>) + Send + 'static,
  ) -> Self {
    let shared: Shared = Arc::default();
    let worker = Arc::clone(&shared);
    std::thread::spawn(move || {
      let helper = work(&worker, &model, helper, &corrected);
      locked(&worker).stopped = true;
      worker.1.notify_all();
      keep(helper);
    });
    Self { shared }
  }

  /// The speaker paused with `text` written, the first `recognised` bytes
  /// of it no longer changing: what is open of it is corrected, unless it
  /// already is.
  pub fn paused(&self, text: &str, recognised: usize) {
    let mut state = locked(&self.shared);
    let open = state.draft.open(text, recognised);
    let wanted = state
      .wanted
      .as_ref()
      .is_some_and(|(_, wanted)| wanted == open);
    if open.trim().is_empty() || wanted || state.draft.answers(open) {
      return;
    }
    state.wanted = Some((state.draft.round(), open.to_owned()));
    self.shared.1.notify_all();
  }

  /// `text` with the corrections made so far.
  pub fn shown(&self, text: &str) -> String {
    locked(&self.shared).draft.corrected(text).0
  }

  /// `text`, the whole dictation, corrected: what is still open is
  /// corrected, waiting for it until `deadline` at most, and left as heard
  /// if its correction has not come by then.
  pub fn finish(&self, text: &str, deadline: Instant) -> String {
    self.paused(text, text.len());
    let mut state = locked(&self.shared);
    loop {
      let (shown, whole) = state.draft.corrected(text);
      let left = deadline.saturating_duration_since(Instant::now());
      if whole || state.stopped || left.is_zero() {
        return shown;
      }
      state = self
        .shared
        .1
        .wait_timeout(state, left)
        .unwrap_or_else(PoisonError::into_inner)
        .0;
    }
  }
}

impl Drop for Corrector {
  fn drop(&mut self) {
    locked(&self.shared).stopped = true;
    self.shared.1.notify_all();
  }
}

/// Corrects what is wanted until the dictation is over, then answers the
/// helper, or none if it failed. An answer takes the line breaks and
/// quotation marks of what was heard ([`as_said`]); one that then fails the
/// lost-word check ([`kept`]), or does not come, leaves the text as heard.
fn work(
  shared: &Shared,
  model: &CleanupModel,
  helper: Option<Transcriber>,
  corrected: &dyn Fn(),
) -> Option<Transcriber> {
  let mut helper = match helper.map_or_else(Transcriber::start_cleanup, Ok) {
    Ok(helper) => helper,
    Err(error) => {
      eprintln!("Could not start the cleanup helper: {error}");
      return None;
    }
  };
  let instructions = model.prompt.instructions.to_owned();
  if let Err(error) = helper.correct_text(model.path.clone(), instructions.clone(), String::new()) {
    eprintln!("Could not load the cleanup model: {error}");
    return None;
  }
  loop {
    let (round, open) = {
      let mut state = locked(shared);
      while state.wanted.is_none() && !state.stopped {
        state = shared.1.wait(state).unwrap_or_else(PoisonError::into_inner);
      }
      match state.wanted.take() {
        Some(wanted) if !state.stopped => wanted,
        _ => return Some(helper),
      }
    };
    let said = open.trim();
    let answer = helper
      .correct_text(
        model.path.clone(),
        instructions.clone(),
        model.prompt.message(said),
      )
      .inspect_err(|error| eprintln!("Could not correct the dictation: {error}"))
      .ok()
      .map(|answer| as_said(said, &answer))
      .filter(|answer| kept(said, answer));
    let lead = &open[..open.len() - open.trim_start().len()];
    let trail = &open[open.trim_end().len()..];
    let answer = format!("{lead}{}{trail}", answer.as_deref().map_or(said, str::trim));
    {
      let mut state = locked(shared);
      if !state.stopped {
        state.draft.answer(round, &open, answer);
      }
    }
    shared.1.notify_all();
    corrected();
  }
}
