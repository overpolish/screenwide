// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a dictation's speech pauses, from the voice activity model's chance
//! of speech in each frame: a stretch of speech runs from a frame that is
//! surely speech until silence has lasted long enough to be a pause.

/// The voice activity model's frame: 32 ms at the protocol's rate.
pub const FRAME: usize = 512;
/// A frame at least this likely to be speech is speech.
const SPEECH: f32 = 0.5;
/// Within speech, a frame less likely than this is silence: between the two,
/// speech carries on, as Silero's own segmenting does.
const SILENCE: f32 = 0.35;
/// Silence that ends a stretch: 800 ms, longer than the gaps inside fluent
/// speech, short enough that the text follows soon after a sentence ends.
const PAUSE_FRAMES: usize = 25;
/// The least speech a stretch holds to be written, 288 ms: anything shorter
/// is a cough or a click, from which Whisper would invent words.
const LEAST_SPEECH_FRAMES: usize = 9;

/// Speech between two pauses, in samples of the dictation: `[start, end)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stretch {
  pub start: usize,
  pub end: usize,
}

#[derive(Debug, Default)]
pub struct Pauses {
  open: Option<Open>,
}

#[derive(Debug)]
struct Open {
  stretch: Stretch,
  speech_frames: usize,
  silent_frames: usize,
}

impl Pauses {
  /// Hears frame `index`, `speech` likely to be speech, answering the
  /// stretch a pause has just ended, if one has.
  pub fn hear(&mut self, index: usize, speech: f32) -> Option<Stretch> {
    let end = (index + 1) * FRAME;
    match &mut self.open {
      None if speech >= SPEECH => {
        self.open = Some(Open {
          stretch: Stretch {
            start: index * FRAME,
            end,
          },
          speech_frames: 1,
          silent_frames: 0,
        });
        None
      }
      None => None,
      Some(open) if speech >= SPEECH => {
        open.stretch.end = end;
        open.speech_frames += 1;
        open.silent_frames = 0;
        None
      }
      Some(open) => {
        if speech < SILENCE {
          open.silent_frames += 1;
        }
        if open.silent_frames < PAUSE_FRAMES {
          return None;
        }
        self.end()
      }
    }
  }

  /// The stretch still open as the dictation ends, if it holds enough
  /// speech to write.
  pub fn end(&mut self) -> Option<Stretch> {
    self
      .open
      .take()
      .filter(|open| open.speech_frames >= LEAST_SPEECH_FRAMES)
      .map(|open| open.stretch)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The stretches `frames` give, each a frame's chance of speech, with
  /// what is still open at the end.
  fn stretches(frames: &[f32]) -> Vec<Stretch> {
    let mut pauses = Pauses::default();
    let mut found: Vec<Stretch> = frames
      .iter()
      .enumerate()
      .filter_map(|(index, &speech)| pauses.hear(index, speech))
      .collect();
    found.extend(pauses.end());
    found
  }

  fn run(speech: f32, frames: usize) -> Vec<f32> {
    vec![speech; frames]
  }

  #[test]
  fn a_pause_ends_a_stretch_and_a_short_gap_does_not() {
    let frames = [
      run(0.9, 20),
      run(0.1, 10),
      run(0.9, 20),
      run(0.1, PAUSE_FRAMES),
      run(0.9, 15),
    ]
    .concat();
    assert_eq!(
      stretches(&frames),
      [
        Stretch {
          start: 0,
          end: 50 * FRAME
        },
        Stretch {
          start: 75 * FRAME,
          end: 90 * FRAME
        }
      ]
    );
  }

  #[test]
  fn too_little_speech_is_not_a_stretch() {
    let frames = [run(0.9, 4), run(0.1, PAUSE_FRAMES), run(0.9, 4)].concat();
    assert!(stretches(&frames).is_empty());
  }

  #[test]
  fn uncertain_frames_carry_speech_on() {
    let frames = [run(0.9, 20), run(0.4, 40), run(0.9, 20)].concat();
    assert_eq!(
      stretches(&frames),
      [Stretch {
        start: 0,
        end: 80 * FRAME
      }]
    );
  }
}
