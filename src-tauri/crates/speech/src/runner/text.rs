// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the text in a picture: the models loaded once, then any number of
//! pictures read.

use std::path::PathBuf;

use screenwide_transcriber::{Job, Reply, TextLine};

use super::Transcriber;

impl Transcriber {
  /// Loads the text detection model at `detection`, the recognition model at
  /// `recognition` and its configuration at `dictionary`, for
  /// [`Self::read_text`].
  pub fn load_text_reader(
    &mut self,
    detection: PathBuf,
    recognition: PathBuf,
    dictionary: PathBuf,
  ) -> Result<(), String> {
    let job = Job::LoadTextReader {
      detection,
      recognition,
      dictionary,
    };
    match self.ask(job, &mut |_| {})? {
      Reply::TextReaderLoaded { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// The lines in the picture in the file `image`, `width` by `height`
  /// pixels of red, green, blue and alpha bytes, in reading order.
  pub fn read_text(
    &mut self,
    image: PathBuf,
    width: u32,
    height: u32,
  ) -> Result<Vec<TextLine>, String> {
    match self.ask(
      Job::ReadText {
        image,
        width,
        height,
      },
      &mut |_| {},
    )? {
      Reply::TextRead { lines, .. } => Ok(lines),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }
}
