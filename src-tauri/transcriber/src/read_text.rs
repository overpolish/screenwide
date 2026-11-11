// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the text in a picture with PP-OCRv6 small (PaddlePaddle,
//! Apache-2.0): a detection model finds each line, and a recognition model
//! reads each one. Both stay loaded for a run of reads.

mod decode;
mod detect;
mod picture;
mod recognise;
#[cfg(test)]
mod tests;

use std::path::Path;

use ort::session::Session;
use ort::value::TensorRef;
use screenwide_transcriber::TextLine;

use decode::Dictionary;
use picture::Picture;
use recognise::Crop;

/// The feature's name in what the helper reports.
const WHAT: &str = "Text recognition";

pub struct TextReader {
  detection: Session,
  recognition: Session,
  dictionary: Dictionary,
}

impl TextReader {
  pub fn load(detection: &Path, recognition: &Path, dictionary: &Path) -> Result<Self, String> {
    let config = std::fs::read_to_string(dictionary)
      .map_err(|error| format!("Could not read the text recognition configuration: {error}"))?;
    Ok(Self {
      detection: crate::session::session(detection, WHAT)?,
      recognition: crate::session::session(recognition, WHAT)?,
      dictionary: Dictionary::from_config(&config)?,
    })
  }

  /// The lines in the picture in the file `image`, `width` by `height`
  /// pixels of red, green, blue and alpha bytes, in reading order.
  pub fn read(&mut self, image: &Path, width: u32, height: u32) -> Result<Vec<TextLine>, String> {
    let rgba =
      std::fs::read(image).map_err(|error| format!("Could not read the picture: {error}"))?;
    let (width, height) = (width as usize, height as usize);
    if width == 0 || height == 0 || rgba.len() != width * height * 4 {
      return Err("The picture's size does not match its pixels".to_owned());
    }
    let picture = Picture {
      rgba: &rgba,
      width,
      height,
    };
    let crops = self
      .find(&picture)?
      .into_iter()
      .map(|found| Crop::of(found, &picture))
      .collect::<Vec<_>>();
    let mut lines = vec![None; crops.len()];
    for (batch, batch_width) in recognise::batches(&crops) {
      let input = recognise::model_input(&picture, &crops, &batch, batch_width);
      let shape = [batch.len(), 3, recognise::HEIGHT, batch_width];
      let input = TensorRef::from_array_view((shape, input.as_slice())).map_err(failed)?;
      let outputs = self
        .recognition
        .run(ort::inputs!["x" => input])
        .map_err(failed)?;
      let (dimensions, chances) = outputs[0].try_extract_tensor::<f32>().map_err(failed)?;
      let (steps, classes) = (dimensions[1] as usize, dimensions[2] as usize);
      if classes != self.dictionary.classes() {
        return Err("The text recognition model and its characters do not match".to_owned());
      }
      for (slot, &index) in batch.iter().enumerate() {
        let row = &chances[slot * steps * classes..(slot + 1) * steps * classes];
        lines[index] = recognise::line(&crops[index], row, (steps, batch_width), &self.dictionary);
      }
    }
    Ok(lines.into_iter().flatten().collect())
  }

  /// Where the lines are, in reading order.
  fn find(&mut self, picture: &Picture<'_>) -> Result<Vec<detect::TextBox>, String> {
    let seen = detect::model_size(picture.width, picture.height);
    let input = detect::model_input(picture, seen);
    let input =
      TensorRef::from_array_view(([1, 3, seen.1, seen.0], input.as_slice())).map_err(failed)?;
    let outputs = self
      .detection
      .run(ort::inputs!["x" => input])
      .map_err(failed)?;
    let (_, chances) = outputs[0].try_extract_tensor::<f32>().map_err(failed)?;
    Ok(detect::boxes(
      chances,
      seen,
      (picture.width, picture.height),
    ))
  }
}

fn failed(error: ort::Error) -> String {
  format!("{WHAT} could not run: {error}")
}
