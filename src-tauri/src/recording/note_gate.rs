// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeps a voice note out of the recording's own microphone track.
//!
//! While a moment's key is held, the recording's microphone is held back
//! rather than written, until the hold is known to be a note or a tap. A
//! note's span is written as silence, so the track keeps its length and its
//! sync; a tap lets everything through as it was, so tapping mid-sentence
//! never cuts the narration. Away from a hold nothing is delayed at all.
//!
//! The same span can be handed to the note itself, when the note is
//! recorded from the recording's own microphone rather than a second one.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::microphone::Buffer;

/// How long a key has to be held for a note rather than a moment alone.
pub(crate) const NOTE_THRESHOLD: Duration = Duration::from_millis(300);

/// Receives the recording microphone's samples within a hold: interleaved,
/// with their channel count and sample rate.
pub(crate) type NoteTee = Arc<dyn Fn(&[f32], u16, u32, Instant) + Send + Sync>;

struct Hold {
  start: Instant,
  end: Option<Instant>,
  tee: Option<NoteTee>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
  /// Still held, and not yet for long enough to tell.
  Open,
  Tap,
  Note,
}

impl Hold {
  fn verdict(&self, now: Instant) -> Verdict {
    let held = self
      .end
      .unwrap_or(now)
      .saturating_duration_since(self.start);
    if held >= NOTE_THRESHOLD {
      Verdict::Note
    } else if self.end.is_some() {
      Verdict::Tap
    } else {
      Verdict::Open
    }
  }

  fn covers(&self, at: Instant) -> bool {
    at >= self.start && self.end.is_none_or(|end| at <= end)
  }
}

static HOLD: Mutex<Option<Hold>> = Mutex::new(None);

fn hold() -> MutexGuard<'static, Option<Hold>> {
  HOLD.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A moment's key went down at `at` with a note to follow. `tee`, when
/// given, takes the recording microphone's samples for the note.
pub(crate) fn press(at: Instant, tee: Option<NoteTee>) {
  *hold() = Some(Hold {
    end: None,
    start: at,
    tee,
  });
}

/// The key came up at `at`.
pub(crate) fn release(at: Instant) {
  if let Some(hold) = hold().as_mut() {
    hold.end.get_or_insert(at);
  }
}

/// The note could not be recorded after all: the recording keeps its own
/// microphone through the hold.
pub(crate) fn cancel() {
  *hold() = None;
}

/// `on_buffer`, behind the gate: what the recording's microphone writer is
/// given in place of the raw stream.
pub(crate) fn gated(
  channels: u16,
  sample_rate: u32,
  on_buffer: Arc<dyn Fn(Buffer) + Send + Sync>,
) -> Arc<dyn Fn(Buffer) + Send + Sync> {
  let waiting = Mutex::new(VecDeque::<Buffer>::new());
  Arc::new(move |buffer: Buffer| {
    let mut waiting = waiting
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (verdict, span) = {
      let hold = hold();
      match hold.as_ref() {
        None => (Verdict::Tap, None),
        Some(hold) => {
          if let Some(tee) = hold
            .tee
            .as_ref()
            .filter(|_| hold.covers(buffer.captured_at))
          {
            tee(&buffer.samples, channels, sample_rate, buffer.captured_at);
          }
          (hold.verdict(Instant::now()), Some((hold.start, hold.end)))
        }
      }
    };
    let held = span.is_some_and(|(start, end)| {
      buffer.captured_at >= start && end.is_none_or(|end| buffer.captured_at <= end)
    });
    match verdict {
      Verdict::Open if held => waiting.push_back(buffer),
      Verdict::Open | Verdict::Tap => {
        for queued in waiting.drain(..) {
          on_buffer(queued);
        }
        on_buffer(buffer);
      }
      Verdict::Note => {
        for queued in waiting.drain(..) {
          on_buffer(silenced(queued));
        }
        on_buffer(if held { silenced(buffer) } else { buffer });
      }
    }
  })
}

fn silenced(mut buffer: Buffer) -> Buffer {
  buffer.samples.fill(0.0);
  buffer
}

#[cfg(test)]
mod tests;
