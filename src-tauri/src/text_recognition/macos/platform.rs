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
  let stride = pixel_buffer.bytes_per_row();
  let base = unsafe { pixel_buffer.base_address_mut() } as *mut u8;
  for row in 0..height as usize {
    let target =
      unsafe { std::slice::from_raw_parts_mut(base.add(row * stride), width as usize * 4) };
    let source = &rgba[row * width as usize * 4..(row + 1) * width as usize * 4];
    for (source, target) in source
      .as_chunks::<4>()
      .0
      .iter()
      .zip(target.as_chunks_mut::<4>().0.iter_mut())
    {
      target.copy_from_slice(&[source[2], source[1], source[0], source[3]]);
    }
  }
  unsafe { pixel_buffer.unlock_lock_base_addr(flags) };

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
