// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use screenwide_transcriber::{Segment, Speech, Transcript, Word, SAMPLE_RATE};
use whisper_rs::{
  DtwMode, DtwModelPreset, DtwParameters, FullParams, SamplingStrategy, WhisperContext,
  WhisperContextParameters, WhisperState, WhisperVadContext, WhisperVadContextParams,
};

/// The voice activity model reads the audio in windows of this many samples:
/// 32 ms at the protocol's rate, which is Silero's own window.
const VAD_WINDOW_SAMPLES: usize = 512;

/// The models last loaded, kept for the requests after them: loading takes
/// far longer than transcribing a short note.
#[derive(Default)]
pub struct Engine {
  loaded: Option<(PathBuf, WhisperContext)>,
  vad: Option<(PathBuf, WhisperVadContext)>,
}

impl Engine {
  pub fn transcribe(
    &mut self,
    model: &Path,
    model_id: &str,
    audio: &Path,
    language: Option<&str>,
    stretches: &[[u64; 2]],
    progress: &mut dyn FnMut(f32),
  ) -> Result<Transcript, String> {
    let samples = read_samples(audio)?;
    let context = self.context(model, model_id)?;

    // A code the engine does not know is detected instead of failing.
    let language = language
      .filter(|code| whisper_rs::get_lang_id(code).is_some())
      .unwrap_or("auto");
    // No stretches means the whole audio, as voice notes have it.
    let stretches: Vec<(usize, usize)> = if stretches.is_empty() {
      vec![(0, samples.len())]
    } else {
      stretches
        .iter()
        .map(|&[start, end]| (start as usize, end as usize))
        .collect()
    };

    let mut all_segments = Vec::new();
    let mut detected: Option<String> = None;
    for (index, &(from, to)) in stretches.iter().enumerate() {
      let slice = &samples[from.min(samples.len())..to.min(samples.len())];
      let mut state = context
        .create_state()
        .map_err(|error| format!("Could not start transcribing: {error}"))?;
      let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
      params.set_language(Some(language));
      params.set_n_threads(threads());
      params.set_token_timestamps(true);
      // Each stretch is its own recording: nothing said in one should steer
      // the next, nor a voice note the one before it.
      params.set_no_context(true);
      // Keeps "[Music]" and the like, said by nobody, out of the text.
      params.set_suppress_nst(true);
      params.set_print_progress(false);
      params.set_print_realtime(false);
      params.set_print_special(false);
      params.set_print_timestamps(false);
      state
        .full(params, slice)
        .map_err(|error| format!("Transcribing failed: {error}"))?;
      let mut segments = segments(context, &state)?;
      // Times are from the stretch's own start; put them back where they are
      // in the whole audio.
      let offset_ms = from as u64 * 1_000 / u64::from(SAMPLE_RATE);
      for segment in &mut segments {
        segment.start_ms += offset_ms;
        segment.end_ms += offset_ms;
        for word in &mut segment.words {
          word.start_ms += offset_ms;
          word.end_ms += offset_ms;
        }
      }
      if detected.is_none() {
        detected = Some(
          whisper_rs::get_lang_str(state.full_lang_id_from_state())
            .unwrap_or(language)
            .to_owned(),
        );
      }
      all_segments.extend(segments);
      // After each stretch, so the app can stop before the next.
      progress((index + 1) as f32 / stretches.len() as f32);
    }
    let language = detected.unwrap_or_else(|| language.to_owned());
    let text = all_segments
      .iter()
      .map(|segment| segment.text.as_str())
      .collect::<Vec<_>>()
      .join(" ");
    Ok(Transcript {
      language,
      model: model_id.to_owned(),
      segments: all_segments,
      text,
    })
  }

  /// How likely each window of the audio is to be speech, telling
  /// `progress` how far it has got. The model runs on the processor: it is
  /// small, and a run over an hour of audio takes seconds there without
  /// waking a GPU.
  ///
  /// The audio is heard a minute at a time, so there is progress to tell.
  /// Each minute is heard from a second before it, so the model has settled
  /// by the time its windows are kept.
  pub fn detect_speech(
    &mut self,
    model: &Path,
    audio: &Path,
    progress: &mut dyn FnMut(f32),
  ) -> Result<Speech, String> {
    const CHUNK_WINDOWS: usize = 1_875;
    const LEAD_IN_WINDOWS: usize = 31;
    let samples = read_samples(audio)?;
    if self.vad.as_ref().is_none_or(|(path, _)| path != model) {
      self.vad = None;
      let path = model
        .to_str()
        .ok_or_else(|| "The voice activity model's path is not valid text".to_owned())?;
      let mut params = WhisperVadContextParams::default();
      params.set_n_threads(threads());
      let context = WhisperVadContext::new(path, params)
        .map_err(|error| format!("Could not load the voice activity model: {error}"))?;
      self.vad = Some((model.to_owned(), context));
    }
    let context = &mut self.vad.as_mut().expect("loaded above").1;
    let windows = samples.len().div_ceil(VAD_WINDOW_SAMPLES);
    let mut probabilities = Vec::with_capacity(windows);
    let mut first = 0;
    while first < windows {
      let lead_in = first.min(LEAD_IN_WINDOWS);
      let from = (first - lead_in) * VAD_WINDOW_SAMPLES;
      let to = ((first + CHUNK_WINDOWS) * VAD_WINDOW_SAMPLES).min(samples.len());
      context
        .detect_speech(&samples[from..to])
        .map_err(|error| format!("Could not listen for speech: {error}"))?;
      let heard = context.probabilities();
      if heard.len() != (to - from).div_ceil(VAD_WINDOW_SAMPLES) {
        return Err("The voice activity model heard in windows of another size".to_owned());
      }
      probabilities.extend_from_slice(&heard[lead_in..]);
      first += CHUNK_WINDOWS;
      progress(first.min(windows) as f32 / windows as f32);
    }
    Ok(Speech {
      probabilities,
      window_samples: VAD_WINDOW_SAMPLES as u32,
    })
  }

  fn context(&mut self, model: &Path, model_id: &str) -> Result<&WhisperContext, String> {
    if self.loaded.as_ref().is_none_or(|(path, _)| path != model) {
      self.loaded = None;
      let path = model
        .to_str()
        .ok_or_else(|| "The model's path is not valid text".to_owned())?;
      // The GPU first; a computer without one it can use transcribes on its
      // processor, more slowly. Both take the model's DTW alignment heads,
      // which sharpen word timestamps where a preset knows the model.
      let context = WhisperContext::new_with_params(path, context_params(model_id, true))
        .or_else(|_| WhisperContext::new_with_params(path, context_params(model_id, false)))
        .map_err(|error| format!("Could not load the model: {error}"))?;
      self.loaded = Some((model.to_owned(), context));
    }
    Ok(&self.loaded.as_ref().expect("loaded above").1)
  }
}

fn threads() -> i32 {
  std::thread::available_parallelism()
    .map_or(4, |count| count.get().min(8))
    .try_into()
    .unwrap_or(4)
}

/// How the model is loaded: on the GPU if `use_gpu`, with its DTW alignment
/// heads. Flash attention stays off, as DTW requires.
fn context_params(model_id: &str, use_gpu: bool) -> WhisperContextParameters<'static> {
  let mut params = WhisperContextParameters::default();
  params.use_gpu(use_gpu);
  params.dtw_parameters(dtw(model_id));
  params
}

/// The DTW alignment heads for a model, which give each token, and so each
/// word, a time closer to the sound than the plain timestamps. A model
/// without a known preset keeps the plain timestamps.
fn dtw(model_id: &str) -> DtwParameters<'static> {
  let preset = match model_id {
    "base" => DtwModelPreset::Base,
    "large-v3-turbo-q8_0" => DtwModelPreset::LargeV3Turbo,
    _ => return DtwParameters::default(),
  };
  DtwParameters {
    mode: DtwMode::ModelPreset {
      model_preset: preset,
    },
    dtw_mem_size: DtwParameters::default().dtw_mem_size,
  }
}

fn read_samples(path: &Path) -> Result<Vec<f32>, String> {
  let bytes = std::fs::read(path).map_err(|error| format!("Could not read the audio: {error}"))?;
  let (samples, _) = bytes.as_chunks::<4>();
  Ok(samples.iter().copied().map(f32::from_ne_bytes).collect())
}

/// The engine's times are in hundredths of a second.
fn ms(centiseconds: i64) -> u64 {
  u64::try_from(centiseconds).unwrap_or(0) * 10
}

fn segments(context: &WhisperContext, state: &WhisperState) -> Result<Vec<Segment>, String> {
  let failed = |error: whisper_rs::WhisperError| format!("Could not read the transcript: {error}");
  // Everything from end-of-text up is a control token: timestamps, language
  // tags and the like, which are not speech.
  let first_special = context.token_eot();
  let mut segments = Vec::new();
  for segment in state.as_iter() {
    let mut words: Vec<Word> = Vec::new();
    // A character can be split across tokens, so a word's bytes are joined
    // before they are read as text.
    let mut bytes: Vec<u8> = Vec::new();
    let mut probabilities: Vec<f32> = Vec::new();
    // A word starts at its first token's DTW time, which follows the sound
    // more closely, or the plain time where there is no DTW.
    let mut span = (0, 0);
    let mut finish = |bytes: &mut Vec<u8>, probabilities: &mut Vec<f32>, span: (u64, u64)| {
      let text = String::from_utf8_lossy(bytes).trim().to_owned();
      if !text.is_empty() {
        #[expect(clippy::cast_precision_loss, reason = "a word has few tokens")]
        let probability = probabilities.iter().sum::<f32>() / probabilities.len() as f32;
        words.push(Word {
          end_ms: span.1,
          probability,
          start_ms: span.0,
          text,
        });
      }
      bytes.clear();
      probabilities.clear();
    };
    for index in 0..segment.n_tokens() {
      let Some(token) = segment.get_token(index) else {
        continue;
      };
      let data = token.token_data();
      if data.id >= first_special {
        continue;
      }
      let piece = token.to_bytes().map_err(failed)?;
      // A token opening with a space starts a word.
      if piece.first() == Some(&b' ') && !bytes.is_empty() {
        finish(&mut bytes, &mut probabilities, span);
      }
      if bytes.is_empty() {
        span.0 = ms(if data.t_dtw >= 0 { data.t_dtw } else { data.t0 });
      }
      span.1 = ms(data.t1);
      bytes.extend_from_slice(piece);
      probabilities.push(data.p);
    }
    finish(&mut bytes, &mut probabilities, span);
    finish_word_ends(&mut words);
    let text = String::from_utf8_lossy(segment.to_bytes().map_err(failed)?)
      .trim()
      .to_owned();
    if text.is_empty() {
      continue;
    }
    segments.push(Segment {
      end_ms: ms(segment.end_timestamp()),
      start_ms: ms(segment.start_timestamp()),
      text,
      words,
    });
  }
  Ok(segments)
}

/// A word ends where the next one begins, never later than its own last
/// token and never before it began.
fn finish_word_ends(words: &mut [Word]) {
  for index in 0..words.len().saturating_sub(1) {
    let end = words[index]
      .end_ms
      .min(words[index + 1].start_ms)
      .max(words[index].start_ms);
    words[index].end_ms = end;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn word(start_ms: u64, end_ms: u64) -> Word {
    Word {
      end_ms,
      probability: 1.0,
      start_ms,
      text: "word".to_owned(),
    }
  }

  #[test]
  fn a_word_ends_where_the_next_begins() {
    // The next word starts before this one's last token runs out.
    let mut words = [word(0, 200), word(100, 300)];
    finish_word_ends(&mut words);
    assert_eq!(words[0].end_ms, 100);
  }

  #[test]
  fn a_word_never_runs_past_its_last_token() {
    // The next word begins after this one's last token; the earlier end wins.
    let mut words = [word(0, 50), word(100, 200)];
    finish_word_ends(&mut words);
    assert_eq!(words[0].end_ms, 50);
  }

  #[test]
  fn a_word_never_ends_before_it_began() {
    // A timestamp that would run backwards is clamped to the word's start.
    let mut words = [word(50, 90), word(30, 200)];
    finish_word_ends(&mut words);
    assert_eq!(words[0].end_ms, 50);
  }
}
