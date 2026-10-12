// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The singing in the system audio, separated by the transcriber with the
//! music separation model, for Studio ducking to move out of the voice's
//! way while the music stays.

use std::path::{Path, PathBuf};

use screenwide_transcriber::SEPARATE_SAMPLE_RATE;

use crate::runner::samples::{decode_stereo_stream, Samples};
use crate::runner::Transcriber;

/// The singing in the `stream`th track of `movie`, both sides interleaved at
/// the protocol's separation rate, found by the model at `model`, telling
/// `progress` how far it has got.
pub(super) fn separate(
  movie: &Path,
  stream: usize,
  model: PathBuf,
  progress: &mut dyn FnMut(f32),
) -> Result<Samples, String> {
  let mix = decode_stereo_stream(movie, stream, SEPARATE_SAMPLE_RATE)?;
  let singing = Samples::scratch();
  Transcriber::start()?.separate_vocals(model, &mix, &singing, progress)?;
  Ok(singing)
}
