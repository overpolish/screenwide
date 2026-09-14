// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Arrow geometry shared by the annotation gesture and the native handles.
//!
//! Annotations are stored in the source's own pixel space. Everything that
//! crosses the FFI is normalised over the whole source image instead, the
//! same 0..1 space `image_x/image_y/image_width/image_height` place the
//! picture in, so the native side can turn a handle into a display point
//! with the transform it already applies to the selection OSC and never has
//! to know how big the source is.

use crate::screenshots::{Annotation, AnnotationPoint, AnnotationShape, AnnotationStyle};

/// The three grips of one arrow, normalised over the source image: the two
/// tips and the point of the curve at `t = 0.5`, which is where the middle
/// handle sits. Six doubles, matching the C `ScreenwidePreviewAnnotation`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativeAnnotationHandles {
  pub(crate) start_x: f64,
  pub(crate) start_y: f64,
  pub(crate) middle_x: f64,
  pub(crate) middle_y: f64,
  pub(crate) end_x: f64,
  pub(crate) end_y: f64,
}

const _: () = assert!(std::mem::size_of::<NativeAnnotationHandles>() == 48);

/// What the pointer does over the picture while a tool is in hand. The select
/// tool hit-tests the arrows already there and lets every other press fall
/// through to the layer; the arrow tool also draws a new one on empty
/// picture. The values are the native `ScreenwideAnnotationMode`.
pub(super) const ANNOTATION_MODE_NONE: u32 = 0;
pub(super) const ANNOTATION_MODE_SELECT: u32 = 1;
pub(super) const ANNOTATION_MODE_ARROW: u32 = 2;

/// The tool name React sends, as a mode. Anything else puts the chrome away.
pub(super) fn annotation_mode(tool: Option<&str>) -> u32 {
  match tool {
    Some("arrow") => ANNOTATION_MODE_ARROW,
    Some("select") => ANNOTATION_MODE_SELECT,
    _ => ANNOTATION_MODE_NONE,
  }
}

/// Where the hover halo is, and how wide. The width is already in the layer's
/// canvas pixels: the native side reports how big the picture is drawn, and
/// the manager knows how many canvas pixels that is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AnnotationHover {
  pub(crate) index: usize,
  pub(crate) layer_id: u64,
  pub(crate) width: f32,
}

/// The halo's width in display points over the pulse, eased the way the
/// ruler's is: three points growing to eight over 160 ms.
pub(super) fn hover_width_points(progress: f64) -> f64 {
  let eased = 1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3);
  3.0 + (8.0 - 3.0) * eased
}

/// The stroke a fresh arrow is drawn with, in output pixels.
pub(super) const NEW_ARROW_WIDTH: f64 = 6.0;

/// The point a quadratic Bézier passes through at `t = 0.5`.
pub(super) fn curve_midpoint(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
) -> AnnotationPoint {
  AnnotationPoint {
    x: 0.25 * start.x + 0.5 * control.x + 0.25 * end.x,
    y: 0.25 * start.y + 0.5 * control.y + 0.25 * end.y,
  }
}

/// The control point that makes the curve pass through `middle` at `t = 0.5`:
/// the inverse of [`curve_midpoint`].
pub(super) fn control_through_midpoint(
  start: AnnotationPoint,
  middle: AnnotationPoint,
  end: AnnotationPoint,
) -> AnnotationPoint {
  AnnotationPoint {
    x: 2.0 * middle.x - (start.x + end.x) / 2.0,
    y: 2.0 * middle.y - (start.y + end.y) / 2.0,
  }
}

/// A point in image-normalised space, in the source's pixels.
pub(super) fn source_point(x: f64, y: f64, source: (u32, u32)) -> AnnotationPoint {
  AnnotationPoint {
    x: x * f64::from(source.0.max(1)),
    y: y * f64::from(source.1.max(1)),
  }
}

/// A point in the source's pixels, normalised over the image.
fn normalised_point(point: AnnotationPoint, source: (u32, u32)) -> (f64, f64) {
  (
    point.x / f64::from(source.0.max(1)),
    point.y / f64::from(source.1.max(1)),
  )
}

/// An arrow in its default dress: the operating system's accent, the tool's
/// stroke, a head at the pointing end, and no bend at all.
pub(super) fn new_arrow(id: String, start: AnnotationPoint, end: AnnotationPoint) -> Annotation {
  Annotation {
    above_camera: false,
    id,
    shape: AnnotationShape::Arrow {
      start,
      control: AnnotationPoint {
        x: (start.x + end.x) / 2.0,
        y: (start.y + end.y) / 2.0,
      },
      end,
    },
    style: AnnotationStyle {
      color: accent_hex(),
      head: crate::screenshots::AnnotationHead::End,
      width: NEW_ARROW_WIDTH,
    },
  }
}

/// The accent every native surface paints with, as `#rrggbb`.
fn accent_hex() -> String {
  let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
  let [red, green, blue] = crate::system_accent::accent_rgb();
  format!(
    "#{:02x}{:02x}{:02x}",
    channel(red),
    channel(green),
    channel(blue)
  )
}

/// Every annotation's grips, in the order the layer stores them.
pub(super) fn annotation_handles(
  annotations: &[Annotation],
  source: (u32, u32),
) -> Vec<NativeAnnotationHandles> {
  annotations
    .iter()
    .map(|annotation| {
      let AnnotationShape::Arrow {
        start,
        control,
        end,
      } = &annotation.shape;
      let (start_x, start_y) = normalised_point(*start, source);
      let (middle_x, middle_y) = normalised_point(curve_midpoint(*start, *control, *end), source);
      let (end_x, end_y) = normalised_point(*end, source);
      NativeAnnotationHandles {
        start_x,
        start_y,
        middle_x,
        middle_y,
        end_x,
        end_y,
      }
    })
    .collect()
}

/// What one layout push tells the native OSC about this layer's arrows.
pub(super) struct AnnotationLayout {
  pub(super) handles: Vec<NativeAnnotationHandles>,
  /// The arrow whose three grips are drawn, or -1 for none.
  pub(super) selected_index: i32,
  pub(super) mode: u32,
}

/// The arrow grips for the pane the OSC is drawn against. The geometry comes
/// from the manager's own output rather than the layout's payload, so a
/// gesture sample and the layout that echoes it publish the same handles.
pub(super) fn annotation_layout(
  manager: &super::state::PreviewManager,
  pane_index: Option<u32>,
  mode: u32,
  selected: Option<&str>,
) -> AnnotationLayout {
  let handles = pane_index
    .and_then(|pane_index| {
      let source = manager.annotation_source(pane_index)?;
      Some(annotation_handles(
        manager.annotations_for(pane_index)?,
        source,
      ))
    })
    .unwrap_or_default();
  let selected_index = pane_index
    .and_then(|pane_index| manager.annotations_for(pane_index))
    .zip(selected)
    .and_then(|(annotations, id)| annotations.iter().position(|item| item.id == id))
    .map_or(-1, |index| i32::try_from(index).unwrap_or(-1));
  AnnotationLayout {
    handles,
    selected_index,
    mode,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const A: AnnotationPoint = AnnotationPoint { x: 10.0, y: 20.0 };
  const C: AnnotationPoint = AnnotationPoint { x: 110.0, y: 60.0 };

  #[test]
  fn the_middle_handle_solves_back_to_the_point_it_was_dragged_to() {
    let middle = AnnotationPoint { x: 40.0, y: 90.0 };
    let control = control_through_midpoint(A, middle, C);
    let solved = curve_midpoint(A, control, C);
    assert!((solved.x - middle.x).abs() < 1e-9, "{solved:?}");
    assert!((solved.y - middle.y).abs() < 1e-9, "{solved:?}");
  }

  #[test]
  fn a_straight_arrow_puts_its_middle_handle_on_the_midpoint() {
    let arrow = new_arrow("a".to_owned(), A, C);
    let AnnotationShape::Arrow {
      start,
      control,
      end,
    } = arrow.shape;
    assert_eq!(control, AnnotationPoint { x: 60.0, y: 40.0 });
    assert_eq!(curve_midpoint(start, control, end), control);
    assert_eq!(arrow.style.width, NEW_ARROW_WIDTH);
    assert_eq!(arrow.style.head, crate::screenshots::AnnotationHead::End);
  }

  #[test]
  fn normalised_and_source_space_are_inverses() {
    let source = (1_920, 1_080);
    let point = source_point(0.25, 0.75, source);
    assert_eq!(point, AnnotationPoint { x: 480.0, y: 810.0 });
    assert_eq!(normalised_point(point, source), (0.25, 0.75));
  }

  #[test]
  fn handles_report_the_curve_point_rather_than_the_control_point() {
    let annotation = new_arrow("a".to_owned(), A, C);
    let bent = Annotation {
      shape: AnnotationShape::Arrow {
        start: A,
        control: AnnotationPoint { x: 60.0, y: 0.0 },
        end: C,
      },
      ..annotation
    };
    let handles = annotation_handles(&[bent], (100, 100));
    assert_eq!(handles.len(), 1);
    assert!((handles[0].start_x - 0.1).abs() < 1e-9);
    // 0.25 * 20 + 0.5 * 0 + 0.25 * 60 = 20
    assert!((handles[0].middle_y - 0.2).abs() < 1e-9, "{handles:?}");
    assert!((handles[0].middle_x - 0.6).abs() < 1e-9, "{handles:?}");
  }

  #[test]
  fn tool_names_read_back_as_modes() {
    assert_eq!(annotation_mode(Some("arrow")), ANNOTATION_MODE_ARROW);
    assert_eq!(annotation_mode(Some("select")), ANNOTATION_MODE_SELECT);
    assert_eq!(annotation_mode(Some("crop")), ANNOTATION_MODE_NONE);
    assert_eq!(annotation_mode(None), ANNOTATION_MODE_NONE);
  }

  #[test]
  fn the_hover_halo_eases_from_three_points_to_eight() {
    assert!((hover_width_points(0.0) - 3.0).abs() < 1e-9);
    assert!((hover_width_points(1.0) - 8.0).abs() < 1e-9);
    // Ease-out cubic is past its halfway point at half the time.
    assert!(hover_width_points(0.5) > 5.5, "{}", hover_width_points(0.5));
  }

  #[test]
  fn the_accent_is_a_six_digit_hex_colour() {
    let hex = accent_hex();
    assert_eq!(hex.len(), 7, "{hex}");
    assert!(hex.starts_with('#'), "{hex}");
    assert!(
      hex[1..].bytes().all(|byte| byte.is_ascii_hexdigit()),
      "{hex}"
    );
  }
}
