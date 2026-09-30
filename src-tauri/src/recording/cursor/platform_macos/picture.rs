// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading what a cursor another application set looks like, for
//! [`super::super::shape`] to tell what it is. The picture is looked at and
//! dropped; none of it reaches the recording.

use objc2_app_kit::{NSBitmapImageRep, NSCursor, NSImageRep};
use objc2_foundation::NSArray;

use super::super::format::CursorStyle;
use super::super::shape::recognise;

/// What `cursor` is by its shape. It is read at twice its point size, as a
/// retina display shows it: fine enough that a thin stem or a finger stays a
/// few pixels wide, small enough to read in a moment.
pub(super) fn recognised(cursor: &NSCursor) -> CursorStyle {
  let image = cursor.image();
  let size = image.size();
  if size.width <= 0.0 || size.height <= 0.0 {
    return CursorStyle::Custom;
  }
  let wanted = size.width * 2.0;
  let representations = image.representations();
  let Some(chosen) = representations
    .iter()
    .filter(|representation| representation.pixelsWide() > 0 && representation.pixelsHigh() > 0)
    .min_by(|a, b| {
      let miss = |representation: &NSImageRep| (representation.pixelsWide() as f64 - wanted).abs();
      miss(a).total_cmp(&miss(b))
    })
  else {
    return CursorStyle::Custom;
  };
  let single = NSArray::from_slice(&[&*chosen as &NSImageRep]);
  let Some(picture) = NSBitmapImageRep::TIFFRepresentationOfImageRepsInArray(&single)
    .and_then(|data| image::load_from_memory(&data.to_vec()).ok())
    .map(image::DynamicImage::into_rgba8)
  else {
    return CursorStyle::Custom;
  };
  // `hotSpot` is in the image's points from its top-left corner; the
  // picture's own width over the point width is the scale it was drawn at.
  let scale = f64::from(picture.width()) / size.width;
  let hotspot = cursor.hotSpot();
  let alpha: Vec<u8> = picture.pixels().map(|pixel| pixel[3]).collect();
  recognise(
    picture.width() as usize,
    picture.height() as usize,
    &alpha,
    (hotspot.x * scale, hotspot.y * scale),
  )
}
