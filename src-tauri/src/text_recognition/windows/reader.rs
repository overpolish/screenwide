// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading text with the bundled PP-OCR models, which read small screen text
//! far better than Windows' own OCR. They run in the helper program, started
//! and loaded as Text Recognition opens, so the models are ready by the time
//! an area is drawn, and let go of when it is put away. Wherever they cannot
//! run, the read falls back to Windows' OCR.

use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, PoisonError};

use screenwide_speech::runner::Transcriber;
use screenwide_transcriber::TextLine;
use tauri::{AppHandle, Manager};

use super::super::{RecognizedCharacter, RecognizedLine, TextRect};

const DETECTION: &str = "ppocr-detection.onnx";
const RECOGNITION: &str = "ppocr-recognition.onnx";
const DICTIONARY: &str = "ppocr-recognition.yml";

enum Slot {
  Loading,
  Ready(Transcriber),
  Unavailable,
}

/// One Text Recognition session's reader: loading, ready or unavailable.
struct Session {
  slot: Mutex<Slot>,
  loaded: Condvar,
}

static SESSION: Mutex<Option<Arc<Session>>> = Mutex::new(None);

/// A bundled model, or beside the program when it runs unbundled during
/// development.
fn model(app: &AppHandle, name: &str) -> Option<PathBuf> {
  let bundled = app
    .path()
    .resource_dir()
    .ok()
    .map(|resources| resources.join("models").join(name));
  let beside = std::env::current_exe()
    .ok()
    .and_then(|program| program.parent().map(|folder| folder.join(name)));
  bundled
    .into_iter()
    .chain(beside)
    .find(|path| path.is_file())
}

/// Starts the helper and loads the models in the background, for the
/// session Text Recognition is opening.
pub(in crate::text_recognition) fn warm(app: &AppHandle) {
  let paths = [DETECTION, RECOGNITION, DICTIONARY].map(|name| model(app, name));
  let [Some(detection), Some(recognition), Some(dictionary)] = paths else {
    eprintln!("Text recognition's models are missing; Windows' OCR reads instead");
    return;
  };
  let session = Arc::new(Session {
    slot: Mutex::new(Slot::Loading),
    loaded: Condvar::new(),
  });
  let previous = SESSION
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .replace(Arc::clone(&session));
  let started = std::thread::Builder::new()
    .name("text-reader".into())
    .spawn(move || {
      drop(previous);
      let reader = Transcriber::start().and_then(|mut reader| {
        reader.load_text_reader(detection, recognition, dictionary)?;
        Ok(reader)
      });
      let mut slot = session.slot.lock().unwrap_or_else(PoisonError::into_inner);
      *slot = match reader {
        Ok(reader) => Slot::Ready(reader),
        Err(error) => {
          eprintln!(
            "Text recognition's models could not load; Windows' OCR reads instead: {error}"
          );
          Slot::Unavailable
        }
      };
      session.loaded.notify_all();
    });
  if let Err(error) = started {
    eprintln!("Could not start text recognition's reader: {error}");
  }
}

/// Lets go of the session's reader, which ends the helper. Off the calling
/// thread, as the helper is waited for as it ends.
pub(in crate::text_recognition) fn release() {
  let session = SESSION
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .take();
  if session.is_some() {
    std::thread::spawn(move || drop(session));
  }
}

/// The lines in the picture, read by the session's models once they have
/// loaded, or nothing where there are none to read with.
pub(in crate::text_recognition) fn read(
  rgba: &[u8],
  width: u32,
  height: u32,
) -> Option<Result<Vec<RecognizedLine>, String>> {
  let session = SESSION
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .clone()?;
  let mut slot = session.slot.lock().unwrap_or_else(PoisonError::into_inner);
  while matches!(*slot, Slot::Loading) {
    slot = session
      .loaded
      .wait(slot)
      .unwrap_or_else(PoisonError::into_inner);
  }
  let Slot::Ready(reader) = &mut *slot else {
    return None;
  };
  let image = std::env::temp_dir().join(format!("screenwide-text-{}.rgba", std::process::id()));
  let lines = std::fs::write(&image, rgba)
    .map_err(|error| format!("Could not hand the picture to the text reader: {error}"))
    .and_then(|()| reader.read_text(image.clone(), width, height));
  let _ = std::fs::remove_file(&image);
  if lines.is_err() {
    // A reader that failed once is not trusted for the rest of the session.
    *slot = Slot::Unavailable;
  }
  Some(lines.map(|lines| {
    lines
      .iter()
      .map(|line| recognized(line, width as f64, height as f64))
      .collect()
  }))
}

/// `line` measured as a share of the picture, its characters numbered as
/// UTF-16 offsets into its text, with no box for a space.
fn recognized(line: &TextLine, width: f64, height: f64) -> RecognizedLine {
  let [left, top, line_width, line_height] = line.bounds.map(f64::from);
  let (y, rect_height) = (top / height, line_height / height);
  let mut offset = 0;
  let characters = line
    .text
    .chars()
    .zip(&line.characters)
    .filter_map(|(character, &[start_x, end_x])| {
      let start = offset;
      offset += character.len_utf16();
      (!character.is_whitespace()).then(|| RecognizedCharacter {
        start,
        end: offset,
        bounds: TextRect {
          x: f64::from(start_x) / width,
          y,
          width: f64::from(end_x - start_x) / width,
          height: rect_height,
        },
      })
    })
    .collect();
  RecognizedLine {
    text: line.text.clone(),
    confidence: line.confidence,
    bounds: TextRect {
      x: left / width,
      y,
      width: line_width / width,
      height: rect_height,
    },
    characters,
  }
}
