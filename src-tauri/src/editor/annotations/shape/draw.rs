// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the native records and the chrome ask of each kind: the slots its
//! draw record carries, the grips it shows, what it offers the snap engine and
//! what a redaction reads of the picture.

impl super::AnnotationShape {
  /// The three points the retained draw record carries. Each kind reads the
  /// slots its own way; [`super::super::native`] documents every reading.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn draw_points(&self, style: &super::super::AnnotationStyle) -> [[f32; 2]; 3] {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => super::super::arrow::native::draw_points(*start, *control, *end),
      Self::Counter { center, angle, .. } => {
        super::super::counter::native::draw_points(*center, *angle)
      }
      Self::Text {
        origin,
        pointer,
        text,
      } => super::super::text::native::draw_points(*origin, pointer, text, style),
      Self::Redact { start, end, .. } => super::super::redact::native::draw_points(*start, *end),
      Self::Highlight { tone, .. } => {
        super::super::highlight::geometry::draw_points(*tone, style.tint)
      }
      Self::Shape { start, end, seed } => {
        super::super::outline::native::draw_points(*start, *end, *seed, style)
      }
      Self::Spotlight { start, end } => {
        super::super::spotlight::native::draw_points(*start, *end, style)
      }
      Self::Draw { points, .. } => super::super::freehand::native::draw_points(points),
      Self::Magnify {
        start, end, loupe, ..
      } => super::super::magnify::native::draw_points(*start, *end, *loupe),
      Self::Sticker {
        center,
        size,
        angle,
        aspect,
        ..
      } => super::super::sticker::native::draw_points(*center, *size, *angle, *aspect),
    }
  }

  /// The `head` the retained draw record carries: which ends of an arrow
  /// have one, nothing for a counter or a shape, and a text box's alignment.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn draw_head(&self, style: &super::super::AnnotationStyle) -> u32 {
    match self {
      Self::Arrow { .. } | Self::Counter { .. } => match style.head {
        super::super::AnnotationHead::None => 0,
        super::super::AnnotationHead::End => 1,
        super::super::AnnotationHead::Both => 2,
      },
      Self::Text { .. } => style.align.raw(),
      Self::Redact { .. }
      | Self::Highlight { .. }
      | Self::Shape { .. }
      | Self::Spotlight { .. }
      | Self::Draw { .. }
      | Self::Magnify { .. } => 0,
      // Which way the picture faces: the shader mirrors it on one.
      Self::Sticker { flip, .. } => u32::from(*flip),
    }
  }

  /// What the retained draw record rasterises as type: a counter's number, a
  /// text box's text, nothing for an arrow.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn draw_text(&self) -> std::borrow::Cow<'_, str> {
    match self {
      Self::Arrow { .. }
      | Self::Redact { .. }
      | Self::Highlight { .. }
      | Self::Shape { .. }
      | Self::Spotlight { .. }
      | Self::Draw { .. }
      | Self::Magnify { .. } => std::borrow::Cow::Borrowed(""),
      // The picture's library id, which the sticker atlas draws it from.
      Self::Sticker { asset, .. } => std::borrow::Cow::Borrowed(asset),
      Self::Counter { value, .. } => std::borrow::Cow::Owned(value.to_string()),
      Self::Text { text, .. } => std::borrow::Cow::Borrowed(text),
    }
  }

  /// The grips the native chrome draws, normalised over the source image. A
  /// stroke appends its fitted line to `paths`, which its grips point into.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn grips(
    &self,
    style: &super::super::AnnotationStyle,
    index: u32,
    source: (u32, u32),
    image_width: f64,
    paths: &mut Vec<[f32; 2]>,
  ) -> super::super::handles::NativeAnnotationHandles {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => super::super::arrow::handles::grips(
        *start,
        *control,
        *end,
        style,
        index,
        source,
        image_width,
      ),
      Self::Counter { center, angle, .. } => {
        super::super::counter::handles::grips(*center, *angle, style, index, source, image_width)
      }
      Self::Text {
        origin,
        pointer,
        text,
      } => super::super::text::handles::grips(
        *origin,
        pointer,
        text,
        style,
        index,
        source,
        image_width,
      ),
      Self::Redact { start, end, .. } => {
        super::super::redact::handles::grips(*start, *end, style.radius, index, source)
      }
      Self::Highlight {
        start, end, bands, ..
      } => super::super::highlight::handles::grips(*start, *end, bands, index, source),
      Self::Shape { start, end, seed } => super::super::outline::handles::grips(
        *start,
        *end,
        *seed,
        style,
        index,
        source,
        image_width,
      ),
      Self::Spotlight { start, end } => {
        super::super::spotlight::handles::grips(*start, *end, style, index, source)
      }
      Self::Draw { points, smooth } => super::super::freehand::handles::grips(
        points,
        *smooth,
        style,
        index,
        source,
        image_width,
        paths,
      ),
      Self::Magnify {
        start,
        end,
        loupe,
        size,
      } => super::super::magnify::handles::grips(
        *start,
        *end,
        (*loupe, *size),
        style,
        index,
        source,
        image_width,
      ),
      Self::Sticker {
        center,
        size,
        angle,
        aspect,
        ..
      } => super::super::sticker::handles::grips(*center, *size, *angle, *aspect, index, source),
    }
  }

  /// The rectangle this annotation offers the snap engine, in the source's
  /// pixels, or `None` for a shape nothing lines up against.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn field_box(
    &self,
    width: f64,
    source_per_size: f64,
  ) -> Option<super::super::snap::SnapBox> {
    match self {
      Self::Arrow { .. } => super::super::arrow::snap::field_box(),
      Self::Counter { center, .. } => {
        super::super::counter::snap::field_box(*center, width, source_per_size)
      }
      Self::Text { origin, text, .. } => {
        super::super::text::snap::field_box(*origin, text, width, source_per_size)
      }
      Self::Redact { start, end, .. } => super::super::redact::snap::field_box(*start, *end),
      Self::Shape { start, end, .. } => super::super::outline::model::field_box(*start, *end),
      Self::Spotlight { start, end } => super::super::spotlight::model::field_box(*start, *end),
      Self::Draw { points, .. } => super::super::freehand::model::field_box(points),
      Self::Magnify { start, end, .. } => super::super::magnify::model::field_box(*start, *end),
      Self::Sticker {
        center,
        size,
        angle,
        aspect,
        ..
      } => super::super::sticker::model::field_box(*center, *size, *angle, *aspect),
      // A highlight lies over the text it marks; nothing lines up against it.
      Self::Highlight { .. } => None,
    }
  }

  /// The fill, flags and parameters a redaction's record carries over the
  /// picture it covers; `None` for every kind that is drawn rather than
  /// applied to the source.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn redaction_fill(
    &self,
    style: &super::super::AnnotationStyle,
    source: super::super::redact::native::RedactSource<'_>,
    held: Option<&super::super::redact::held::HeldFill>,
  ) -> Option<super::super::redact::native::RedactFill> {
    match self {
      Self::Redact { start, end, seed } => Some(super::super::redact::native::fill(
        *start, *end, *seed, style, source, held,
      )),
      Self::Arrow { .. }
      | Self::Counter { .. }
      | Self::Text { .. }
      | Self::Highlight { .. }
      | Self::Shape { .. }
      | Self::Spotlight { .. }
      | Self::Draw { .. }
      | Self::Magnify { .. }
      | Self::Sticker { .. } => None,
    }
  }
}
