// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading what a cursor another application set looks like: for
//! [`super::super::shape`] to tell what it is, and, for one that is none of
//! the system's, to keep it beside the recording so it is drawn as it was.

use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;

use image::RgbaImage;
use objc2::rc::Retained;
use objc2_app_kit::{NSBitmapImageRep, NSCursor, NSImageRep};
use objc2_foundation::NSArray;

use super::super::shape::recognise;
use super::CursorPicture;
use screenwide_records::cursor::CursorStyle;

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
  let Some(picture) = read(cursor, |representation| {
    -(representation.pixelsWide() as f64 - wanted).abs()
  }) else {
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

/// `cursor`'s picture to keep, at the largest size the app gave it, named
/// for its content; `None` where it has no bitmap to read.
pub(super) fn kept(cursor: &NSCursor) -> Option<CursorPicture> {
  let picture = read(cursor, |representation| representation.pixelsWide() as f64)?;
  let mut hasher = DefaultHasher::new();
  hasher.write_u32(picture.width());
  hasher.write_u32(picture.height());
  hasher.write(picture.as_raw());
  let mut png = Vec::new();
  picture
    .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
    .ok()?;
  Some(CursorPicture {
    name: format!("{:016x}", hasher.finish()),
    png: Arc::from(png),
  })
}

/// The bitmap of `cursor`'s picture that `fit` scores highest.
fn read(cursor: &NSCursor, fit: impl Fn(&NSImageRep) -> f64) -> Option<RgbaImage> {
  let representations = cursor.image().representations();
  let chosen: Retained<NSImageRep> = representations
    .iter()
    .filter(|representation| representation.pixelsWide() > 0 && representation.pixelsHigh() > 0)
    .max_by(|a, b| fit(a).total_cmp(&fit(b)))?;
  let single = NSArray::from_slice(&[&*chosen]);
  NSBitmapImageRep::TIFFRepresentationOfImageRepsInArray(&single)
    .and_then(|data| image::load_from_memory(&data.to_vec()).ok())
    .map(image::DynamicImage::into_rgba8)
}
