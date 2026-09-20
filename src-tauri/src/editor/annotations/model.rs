// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawn annotations carried by an editor layer.
//!
//! Annotations live in the source's own pixel space, the same space the crop
//! is expressed in, so they stay glued to the picture while the frame is
//! moved, resized or re-cropped. The compositor draws them, which is what
//! keeps the editor's preview and the exported PNG the same image.

use super::reveal::AnnotationReveal;
use super::shape::AnnotationShape;
use serde::{Deserialize, Serialize};

/// A point in the screenshot source's pixel space.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationPoint {
  pub x: f64,
  pub y: f64,
}

/// Which ends of an arrow carry a head.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnotationHead {
  None,
  #[default]
  End,
  Both,
}

/// How an annotation is painted. The width is in output pixels, so an
/// annotation keeps its weight on the canvas rather than growing with the
/// picture.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationStyle {
  /// `#rrggbb` or `#rrggbbaa`, straight alpha.
  pub color: String,
  #[serde(default)]
  pub head: AnnotationHead,
  pub width: f64,
}

/// One drawn annotation, independent of its workspace and timing.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
  /// Whether the annotation is drawn over the camera bubble rather than under
  /// it. Screenshots have no bubble; the recording kernel will honour this.
  #[serde(default)]
  pub above_camera: bool,
  /// Whether a timed annotation draws itself in at the start of its clip and
  /// undraws at the end. Stills have no clip to animate over, so they ignore
  /// it and draw whole.
  #[serde(default = "default_animated")]
  pub animated: bool,
  pub id: String,
  /// How much of the path is showing this frame. Derived from the clip's
  /// bounds and the frame's source time every frame and never stored, so a
  /// scrub backwards lands on exactly the frame playing forwards drew.
  #[serde(skip)]
  pub reveal: AnnotationReveal,
  pub shape: AnnotationShape,
  pub style: AnnotationStyle,
}

/// An annotation animates unless a document from before the reveal, or the
/// editor, says otherwise.
fn default_animated() -> bool {
  true
}

/// The colour a fresh annotation is drawn in before anything has been chosen:
/// the palette's yellow, which reads as an annotation on almost any screenshot
/// where the accent would sometimes be the very colour being pointed at. The
/// twin of `ANNOTATION_SWATCHES` in
/// `src/features/editor/annotation-palette.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(super) const NEW_ANNOTATION_COLOR: &str = "#ffcc00";

/// An annotation colour as straight RGBA, from `#rrggbb` or `#rrggbbaa`.
/// An unreadable colour is fully transparent rather than an error: one bad
/// annotation must not cost the whole composition.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn annotation_colour(value: &str) -> [f32; 4] {
  let value = value.strip_prefix('#').unwrap_or(value);
  if !matches!(value.len(), 6 | 8) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
    return [0.0; 4];
  }
  let channel = |start: usize| {
    u8::from_str_radix(&value[start..start + 2], 16).map_or(0.0, |byte| f32::from(byte) / 255.0)
  };
  [
    channel(0),
    channel(2),
    channel(4),
    if value.len() == 8 { channel(6) } else { 1.0 },
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_both_hex_lengths() {
    assert_eq!(annotation_colour("#ff0000"), [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(annotation_colour("#00ff0080")[3], 128.0 / 255.0);
    assert_eq!(annotation_colour("nonsense"), [0.0; 4]);
  }

  #[test]
  fn round_trips_an_arrow_in_camel_case() {
    let annotation = Annotation {
      above_camera: false,
      animated: true,
      id: "a".to_owned(),
      reveal: AnnotationReveal::default(),
      shape: AnnotationShape::Arrow {
        start: AnnotationPoint { x: 1.0, y: 2.0 },
        control: AnnotationPoint { x: 3.0, y: 4.0 },
        end: AnnotationPoint { x: 5.0, y: 6.0 },
      },
      style: AnnotationStyle {
        color: "#ff0000".to_owned(),
        head: AnnotationHead::Both,
        width: 8.0,
      },
    };
    let json = serde_json::to_string(&annotation).unwrap();
    assert!(json.contains("\"aboveCamera\":false"), "{json}");
    assert!(json.contains("\"animated\":true"), "{json}");
    // The reveal is this frame's state, not the document's.
    assert!(!json.contains("reveal"), "{json}");
    assert!(json.contains("\"kind\":\"arrow\""), "{json}");
    assert!(json.contains("\"head\":\"both\""), "{json}");
    assert_eq!(
      serde_json::from_str::<Annotation>(&json).unwrap(),
      annotation
    );
  }

  #[test]
  fn defaults_the_optional_fields() {
    let annotation: Annotation = serde_json::from_str(
      r##"{"id":"a","shape":{"kind":"arrow","start":{"x":0,"y":0},
        "control":{"x":1,"y":1},"end":{"x":2,"y":2}},
        "style":{"color":"#fff000","width":4}}"##,
    )
    .unwrap();
    assert!(!annotation.above_camera);
    // A document written before annotations could animate still animates.
    assert!(annotation.animated);
    assert!(annotation.reveal.is_whole());
    assert_eq!(annotation.style.head, AnnotationHead::End);
  }
}
