// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The faces the engine sets type in: the app's Inter for annotations, and
//! for captions any family the app carries or the system has installed,
//! upright or italic, each made once and kept.

use super::*;

impl Engine {
  /// `face` as the engine sets it, made the first time it is asked for. A
  /// family no longer installed is set in Inter at the weight asked for.
  pub(in crate::engine::type_device) fn face(&self, face: &TypeFace) -> Result<Face, String> {
    if let Some(known) = self.faces.borrow().get(face) {
      return Ok(known.clone());
    }
    let made = self
      .make_face(face)
      .map_err(|error| format!("Windows could not create the caption font: {error}"))?;
    self.faces.borrow_mut().insert(face.clone(), made.clone());
    Ok(made)
  }

  fn make_face(&self, face: &TypeFace) -> windows::core::Result<Face> {
    let weight = f32::from(face.weight);
    // The app's own copy of a family before the system's, as the font list
    // offers it.
    let found = match face.family.as_deref() {
      Some(family) => match Self::found(&self.collection, family, face)? {
        Some(metrics) => Some((false, metrics)),
        None => Self::found(&self.system, family, face)?.map(|metrics| (true, metrics)),
      },
      None => None,
    };
    let (system, family, (ascent, descent)) = match found {
      Some((system, metrics)) => (
        system,
        HSTRING::from(face.family.as_deref().unwrap_or_default()),
        metrics,
      ),
      None => (false, HSTRING::from("Inter"), self.inter_metrics),
    };
    let collection = if system {
      &self.system
    } else {
      &self.collection
    };
    let reference = text_format(
      &self.factory,
      collection,
      &family,
      (weight, face.italic),
      REFERENCE_SIZE,
      ascent * REFERENCE_SIZE,
      descent * REFERENCE_SIZE,
    )?;
    Ok(Face {
      reference,
      ascent,
      descent,
      system,
      family,
      weight,
      italic: face.italic,
    })
  }

  /// The ascent and descent in ems of `family` in `collection` at its
  /// nearest to `face`'s weight and slant, or none where it has no such
  /// family.
  fn found(
    collection: &IDWriteFontCollection2,
    family: &str,
    face: &TypeFace,
  ) -> windows::core::Result<Option<(f64, f64)>> {
    unsafe {
      let name = HSTRING::from(family);
      let mut index = 0_u32;
      let mut exists = windows::core::BOOL::default();
      collection.FindFamilyName(PCWSTR(name.as_ptr()), &mut index, &mut exists)?;
      if !exists.as_bool() {
        return Ok(None);
      }
      let font = collection.GetFontFamily(index)?.GetFirstMatchingFont(
        DWRITE_FONT_WEIGHT(i32::from(face.weight)),
        DWRITE_FONT_STRETCH_NORMAL,
        if face.italic {
          DWRITE_FONT_STYLE_ITALIC
        } else {
          DWRITE_FONT_STYLE_NORMAL
        },
      )?;
      let mut metrics = DWRITE_FONT_METRICS::default();
      font.GetMetrics(&mut metrics);
      let em = f64::from(metrics.designUnitsPerEm.max(1));
      Ok(Some((
        f64::from(metrics.ascent) / em,
        f64::from(metrics.descent) / em,
      )))
    }
  }

  /// `face` at `size` pixels to the em, placed as [`text_format`] places
  /// it.
  pub(in crate::engine::type_device) fn format(
    &self,
    face: &Face,
    size: f64,
    ascent: f64,
    descent: f64,
  ) -> Result<IDWriteTextFormat3, String> {
    let collection = if face.system {
      &self.system
    } else {
      &self.collection
    };
    text_format(
      &self.factory,
      collection,
      &face.family,
      (face.weight, face.italic),
      size,
      ascent,
      descent,
    )
    .map_err(|error| format!("Windows could not create the annotation font: {error}"))
  }
}

/// `family` at a weight, italic where asked, and `size` pixels to the em,
/// each line's baseline `ascent` below its top and the next line
/// `ascent + descent` below that: the spacing GDI's text cell set, which the
/// layouts here are placed by.
fn text_format(
  factory: &IDWriteFactory6,
  collection: &IDWriteFontCollection2,
  family: &HSTRING,
  (weight, italic): (f32, bool),
  size: f64,
  ascent: f64,
  descent: f64,
) -> windows::core::Result<IDWriteTextFormat3> {
  // The typographic family model picks a family's italic face, or its
  // italic axis, by the `ital` value.
  let axes = [
    DWRITE_FONT_AXIS_VALUE {
      axisTag: DWRITE_FONT_AXIS_TAG_WEIGHT,
      value: weight,
    },
    DWRITE_FONT_AXIS_VALUE {
      axisTag: DWRITE_FONT_AXIS_TAG_ITALIC,
      value: if italic { 1.0 } else { 0.0 },
    },
  ];
  unsafe {
    let format = factory.CreateTextFormat(
      PCWSTR(family.as_ptr()),
      collection,
      &axes,
      size as f32,
      w!("en-us"),
    )?;
    format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
    format.SetLineSpacing(&DWRITE_LINE_SPACING {
      method: DWRITE_LINE_SPACING_METHOD_UNIFORM,
      height: (ascent + descent) as f32,
      baseline: ascent as f32,
      leadingBefore: 0.0,
      fontLineGapUsage: DWRITE_FONT_LINE_GAP_USAGE_DEFAULT,
    })?;
    Ok(format)
  }
}
