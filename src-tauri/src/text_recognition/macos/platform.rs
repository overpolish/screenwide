// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::{cv, ns, vn};

use super::super::{RecognizedCharacter, RecognizedLine, TextRect};

fn text_rect(bounds: cidre::cg::Rect) -> TextRect {
  TextRect {
    height: bounds.size.height,
    width: bounds.size.width,
    x: bounds.origin.x,
    // Vision is normalized from the bottom-left; the UI is top-left.
    y: 1.0 - bounds.origin.y - bounds.size.height,
  }
}

fn clamp_rect(rect: TextRect, bounds: TextRect) -> Option<TextRect> {
  let left = rect.x.max(bounds.x);
  let top = rect.y.max(bounds.y);
  let right = (rect.x + rect.width).min(bounds.x + bounds.width);
  let bottom = (rect.y + rect.height).min(bounds.y + bounds.height);
  (right > left && bottom > top).then_some(TextRect {
    height: bottom - top,
    width: right - left,
    x: left,
    y: top,
  })
}

/// `vImage_Buffer`: rows of pixels in memory.
#[repr(C)]
struct ImageBuffer {
  data: *mut std::ffi::c_void,
  height: usize,
  width: usize,
  row_bytes: usize,
}

#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
  fn vImagePermuteChannels_ARGB8888(
    source: *const ImageBuffer,
    destination: *const ImageBuffer,
    permute_map: *const u8,
    flags: u32,
  ) -> isize;
}

/// Vision reads BGRA, so the picture's red and blue are swapped into the
/// locked `buffer` row by row by vImage, which is vectorized: a loop of our
/// own over every pixel is several times slower, and slower still in a dev
/// build, where this crate is not optimized.
fn to_bgra(rgba: &[u8], width: u32, height: u32, buffer: &mut cv::PixelBuf) -> Result<(), String> {
  let (width, height) = (width as usize, height as usize);
  if rgba.len() < width * height * 4 {
    return Err("The selected image is smaller than its size".to_owned());
  }
  let source = ImageBuffer {
    data: rgba.as_ptr().cast_mut().cast(),
    height,
    width,
    row_bytes: width * 4,
  };
  let destination = ImageBuffer {
    data: unsafe { buffer.base_address_mut() }.cast(),
    height,
    width,
    row_bytes: buffer.bytes_per_row(),
  };
  // Each destination channel names the source channel it takes.
  let map = [2_u8, 1, 0, 3];
  // SAFETY: both buffers span `height` rows of `row_bytes`, the source is only
  // read, and the destination is the pixel buffer locked by the caller.
  match unsafe { vImagePermuteChannels_ARGB8888(&source, &destination, map.as_ptr(), 0) } {
    0 => Ok(()),
    error => Err(format!("Could not prepare the image for Vision ({error})")),
  }
}

pub fn recognize(rgba: &[u8], width: u32, height: u32) -> Result<Vec<RecognizedLine>, String> {
  // Vision hands its requests, results and candidates over autoreleased, and
  // recognition runs on a pool thread that never drains one of its own.
  let _pool = cidre::objc::AutoreleasePoolPage::push();
  let mut pixel_buffer = cv::PixelBuf::new(
    width as usize,
    height as usize,
    cv::PixelFormat::_32_BGRA,
    None,
  )
  .map_err(|error| error.to_string())?;
  let flags = cv::pixel_buffer::LockFlags::DEFAULT;
  unsafe { pixel_buffer.lock_base_addr(flags) }
    .result()
    .map_err(|error| error.to_string())?;
  let converted = to_bgra(rgba, width, height, &mut pixel_buffer);
  unsafe { pixel_buffer.unlock_lock_base_addr(flags) };
  converted?;

  let mut request = vn::RecognizeTextRequest::new();
  request.set_revision(vn::RecognizeTextRequest::REVISION_3);
  request.set_recognition_level(vn::RequestTextRecognitionLevel::Accurate);
  request.set_automatically_detects_lang(true);
  // Correction improves prose, but can silently rewrite punctuation and code.
  request.set_uses_lang_correction(false);
  let handler = vn::ImageRequestHandler::with_cv_pixel_buf(&pixel_buffer, None)
    .ok_or_else(|| "Vision could not read the selected image".to_owned())?;
  let requests = ns::Array::<vn::Request>::from_slice(&[&request]);
  handler
    .perform(&requests)
    .map_err(|error| error.map_or_else(|| "Vision failed".to_owned(), ToString::to_string))?;

  let mut lines = Vec::new();
  if let Some(observations) = request.results() {
    for observation in observations.iter() {
      let candidates = observation.top_candidates(1);
      let Some(candidate) = candidates.first() else {
        continue;
      };
      let text = candidate.string().to_string();
      let line_bounds = text_rect(observation.bounding_box());
      let mut utf16_offset = 0;
      let mut characters = text
        .chars()
        .filter_map(|character| {
          let length = character.len_utf16();
          let start = utf16_offset;
          utf16_offset += length;
          if character.is_whitespace() {
            return None;
          }
          candidate
            .bounding_box_for_range(ns::Range::new(start, length))
            .ok()
            .and_then(|observation| clamp_rect(text_rect(observation.bounding_box()), line_bounds))
            .map(|bounds| RecognizedCharacter {
              bounds,
              end: utf16_offset,
              start,
            })
        })
        .collect::<Vec<_>>();
      split_shared_boxes(&mut characters);
      lines.push(RecognizedLine {
        bounds: line_bounds,
        characters,
        confidence: candidate.confidence(),
        text,
      });
    }
  }
  lines.sort_by(|a, b| {
    a.bounds
      .y
      .total_cmp(&b.bounds.y)
      .then_with(|| a.bounds.x.total_cmp(&b.bounds.x))
  });
  Ok(lines)
}

/// Vision answers a character's range with the box of the whole word it is
/// in, so a drag could only stop at a word's edges. Each run of characters
/// sharing one box takes an even share of it, as Windows' OCR words do.
fn split_shared_boxes(characters: &mut [RecognizedCharacter]) {
  let mut start = 0;
  while start < characters.len() {
    let bounds = characters[start].bounds;
    let end = characters[start..]
      .iter()
      .position(|character| character.bounds != bounds)
      .map_or(characters.len(), |length| start + length);
    let count = (end - start) as f64;
    for (index, character) in characters[start..end].iter_mut().enumerate() {
      character.bounds.width = bounds.width / count;
      character.bounds.x = bounds.x + bounds.width * index as f64 / count;
    }
    start = end;
  }
}

#[cfg(test)]
mod tests;
