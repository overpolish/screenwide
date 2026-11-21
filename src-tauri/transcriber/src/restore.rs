// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Restore voice: a voice restored by UniPASE (Xiaobin Rong et al., Nanjing
//! University; Apache-2.0). It cleans up the voice that was recorded rather
//! than speaking it again, so it keeps the speaker's own pitch where a
//! model that rebuilds the voice from what it understood of it wobbles.
//!
//! It runs as two models. The first (`restore`, a speech model that has
//! learnt to hear past damage, with an adapter and a vocoder) restores the
//! voice at 16 kHz on the GPU. A piece that comes back silent from the GPU,
//! as DirectML hands back work Windows reset the GPU under, is restored
//! again on the CPU (`came_back_empty`). The second (`widen`, a TF-GridNet)
//! adds the band above 8 kHz that a headset's microphone never sent, at
//! 48 kHz; its recurrent layers run far faster on the CPU than on a GPU
//! through WebGPU, so it stays there.
//!
//! The voice goes through in pieces of about ten seconds (`pieces`). The
//! first model hears eight seconds of speech before each piece and two
//! after: heard cold, it wobbles on the first words. Before the very first
//! piece there is nothing to hear, so it hears speech from later on.

mod output;
mod pieces;
mod widen;

use std::path::Path;

use ort::value::TensorRef;
use screenwide_transcriber::{CLEAN_SAMPLE_RATE, SAMPLE_RATE};

use self::output::Output;
use self::pieces::{Kept, Piece};
use self::widen::{Restored, Widening};
use crate::session::{gpu_session, Fusion};
use crate::shared_gpu::{came_back_empty, On, Shared};

const RATE: usize = SAMPLE_RATE as usize;
/// Output samples for each input sample: 16 kHz in, 48 kHz out.
const UPSAMPLE: usize = (CLEAN_SAMPLE_RATE / SAMPLE_RATE) as usize;
/// Speech the first model hears before a piece, and after it.
const LEAD: usize = 8 * RATE;
const TAIL: usize = 2 * RATE;
/// Of the restored voice either side of a piece, what the second model
/// hears with it, so the band it adds runs on across the join.
const WIDEN_CONTEXT: usize = RATE;
/// The first model takes 20 ms frames with a 25 ms window: a whole number
/// of frames and the window's extra 5 ms.
const FRAME: usize = 320;
const WINDOW_EXTRA: usize = 80;
/// The feature's name in what the helper reports.
const WHAT: &str = "Restore voice";

/// Restores the `stretches` of the mono samples at `audio`, at the
/// protocol's sample rate, into `output` at the cleaning rate, three samples
/// for each one in and lined up with them, with silence between, by the
/// models at `restorer` and `widener`. What a job stopped part-way left in
/// `output` is kept, and the job goes on from the piece it was restoring.
/// Tells `progress` how far it has got.
pub fn restore_speech(
  (restorer, widener): (&Path, &Path),
  audio: &Path,
  stretches: &[[u64; 2]],
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let samples = read_samples(audio)?;
  let kept = Kept::new(&samples, stretches);
  let pieces = kept.pieces();
  // The pieces fall in the same places every time, so the last one that
  // was started is where the job before was stopped.
  let left_at = Output::left_at(output);
  let first = pieces
    .iter()
    .rposition(|piece| piece.samples.start * UPSAMPLE <= left_at)
    .unwrap_or(0);
  let from = pieces.get(first).map_or(0, |piece| piece.samples.start);
  let mut out = Output::from(output, from.min(left_at / UPSAMPLE) * UPSAMPLE)?;
  if first < pieces.len() {
    let todo = &pieces[first..];
    let mut writer = Writer {
      out: &mut out,
      pieces: todo,
      written: 0,
      done: pieces[..first]
        .iter()
        .map(|piece| piece.samples.len())
        .sum(),
      work: kept.total().max(1),
      progress,
    };
    std::thread::scope(|scope| -> Result<(), String> {
      let mut restore = Shared::open(restorer, WHAT, || gpu_session(restorer, WHAT, Fusion::On));
      let mut widening = Widening::start(scope, widener)?;
      for (index, piece) in todo.iter().enumerate() {
        widening.send(restore_piece(&mut restore, &kept, piece, index)?)?;
        while let Some(voice) = widening.next(false) {
          writer.write(voice?)?;
        }
      }
      widening.close();
      while let Some(voice) = widening.next(true) {
        writer.write(voice?)?;
      }
      Ok(())
    })?;
  }
  out.silence_until(samples.len() * UPSAMPLE)?;
  out.finish()
}

/// Writes the widened pieces in order as they come, telling `progress`.
struct Writer<'a, 'p> {
  out: &'a mut Output,
  pieces: &'a [Piece],
  written: usize,
  done: usize,
  work: usize,
  progress: &'p mut dyn FnMut(f32),
}

impl Writer<'_, '_> {
  fn write(&mut self, voice: Vec<f32>) -> Result<(), String> {
    let piece = &self.pieces[self.written];
    self.out.silence_until(piece.samples.start * UPSAMPLE)?;
    let stretch = piece.stretch.start * UPSAMPLE..piece.stretch.end * UPSAMPLE;
    self.out.piece(&voice, &stretch)?;
    self.written += 1;
    self.done += piece.samples.len();
    #[expect(clippy::cast_precision_loss, reason = "a fraction")]
    (self.progress)(self.done as f32 / self.work as f32);
    Ok(())
  }
}

/// `piece` restored, with the restored voice either side of it that the
/// second model hears it with.
fn restore_piece(
  restore: &mut Shared,
  kept: &Kept<'_>,
  piece: &Piece,
  index: usize,
) -> Result<Restored, String> {
  let length = piece.samples.len();
  let mut lead = kept.before(piece.samples.start, LEAD);
  if lead.len() < LEAD {
    let mut later = kept.after(piece.samples.end, LEAD - lead.len());
    later.append(&mut lead);
    lead = later;
  }
  let tail = kept.after(piece.samples.end, TAIL);
  let mut heard = Vec::with_capacity(lead.len() + length + tail.len() + FRAME);
  heard.extend_from_slice(&lead);
  heard.extend_from_slice(kept.samples(piece.samples.clone()));
  heard.extend_from_slice(&tail);
  let unpadded = heard.len();
  let frames = unpadded.saturating_sub(WINDOW_EXTRA).div_ceil(FRAME).max(1);
  heard.resize(frames * FRAME + WINDOW_EXTRA, 0.0);
  let mut restored = run_restore(restore, &heard)?;
  restored.resize(unpadded, 0.0);
  let from = lead.len().saturating_sub(WIDEN_CONTEXT);
  let to = (lead.len() + length + WIDEN_CONTEXT).min(restored.len());
  restored.truncate(to);
  Ok(Restored {
    index,
    voice: restored.split_off(from),
    start: lead.len() - from,
    length,
  })
}

fn run_restore(restore: &mut Shared, heard: &[f32]) -> Result<Vec<f32>, String> {
  let failed = |error: ort::Error| format!("{WHAT} could not run: {error}");
  restore.run(|session, on| {
    let speech = TensorRef::from_array_view(([1_usize, heard.len()], heard)).map_err(failed)?;
    let outputs = session
      .run(ort::inputs!["speech" => speech])
      .map_err(failed)?;
    let (_, restored) = outputs["restored"]
      .try_extract_tensor::<f32>()
      .map_err(failed)?;
    if on == On::Gpu && is_silent(restored) && !is_silent(heard) {
      return Err(came_back_empty(WHAT));
    }
    Ok(restored.to_vec())
  })
}

/// Whether `samples` hold nothing at all, quieter than any recording's own
/// noise: what DirectML hands back from a GPU Windows reset.
fn is_silent(samples: &[f32]) -> bool {
  samples.iter().all(|sample| sample.abs() < 1e-6)
}

fn read_samples(audio: &Path) -> Result<Vec<f32>, String> {
  let bytes = std::fs::read(audio).map_err(|error| format!("Could not read the audio: {error}"))?;
  Ok(
    bytes
      .as_chunks::<4>()
      .0
      .iter()
      .map(|&sample| f32::from_le_bytes(sample))
      .collect(),
  )
}
