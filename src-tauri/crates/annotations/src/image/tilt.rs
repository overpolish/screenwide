// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How an image is turned in depth, and where that puts its own frame's
//! points.
//!
//! An image is turned three ways, as the screen is: about its own across and
//! down axes in depth, here, and flat about its middle by its `angle`. The
//! flat turn comes last, so a point of the picture's own frame is tilted in
//! depth first, still in that frame, and then turned and placed as an upright
//! picture's point is. The tilt is seen from in front of the picture, from
//! the screen's own distance in lengths of the picture's longer side, so a
//! picture of any size foreshortens alike. The twin of the image shader's
//! `annotation_image_untilt`.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use super::model::{from_local, to_local};
use crate::AnnotationPoint;
use screenwide_scenes::rotation::{apply, inverted, turned_homography, Homography};
use screenwide_scenes::SceneRotation;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How far an image is turned in depth, in degrees: about its own across
/// axis in `x`, tipping its top towards the viewer as it grows, and about its
/// own up axis in `y`, swinging its right side away. Flat where both are
/// zero. Its flat turn is the image's `angle`.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ImageTilt {
  pub x: f64,
  pub y: f64,
}

impl ImageTilt {
  pub fn is_flat(self) -> bool {
    self.x == 0.0 && self.y == 0.0
  }

  pub(crate) fn is_finite(self) -> bool {
    self.x.is_finite() && self.y.is_finite()
  }

  /// The tilt a draw record carries, in radians.
  pub fn radians(self) -> [f32; 2] {
    [self.x.to_radians() as f32, self.y.to_radians() as f32]
  }

  /// The tilt a draw record carried, from its radians.
  pub fn from_radians([x, y]: [f32; 2]) -> Self {
    Self {
      x: f64::from(x).to_degrees(),
      y: f64::from(y).to_degrees(),
    }
  }

  /// What carries a point of the picture's own frame, across and down from
  /// its middle, to where the tilt draws it in that frame; `None` where the
  /// picture is flat. `longer` is its longer side, in the same units.
  fn homography(self, longer: f64) -> Option<Homography> {
    (!self.is_flat()).then(|| {
      turned_homography(
        SceneRotation {
          x: self.x,
          y: self.y,
          z: 0.0,
        },
        (0.0, 0.0),
        longer,
      )
    })
  }

  /// Where the tilt draws `local`, a point of the picture's own frame: far
  /// off every picture where it lands behind the eye.
  pub(crate) fn tilted(self, longer: f64, local: AnnotationPoint) -> AnnotationPoint {
    let Some(turn) = self.homography(longer) else {
      return local;
    };
    let (x, y) = apply(turn, (local.x, local.y)).unwrap_or((f64::MAX, f64::MAX));
    AnnotationPoint { x, y }
  }

  /// The point of the picture's own frame the tilt draws at `drawn`, or
  /// `None` where nothing of the picture is drawn there: past its horizon,
  /// or with the picture seen edge on.
  pub(crate) fn untilted(self, longer: f64, drawn: AnnotationPoint) -> Option<AnnotationPoint> {
    let Some(turn) = self.homography(longer) else {
      return Some(drawn);
    };
    let back = inverted(turn)?;
    let (x, y) = apply(back, (drawn.x, drawn.y))?;
    // A point behind the eye can map back onto the picture too; only one the
    // picture actually covers in front of it counts.
    apply(turn, (x, y))?;
    Some(AnnotationPoint { x, y })
  }
}

/// Where the picture's own point `local` is drawn: tilted in depth, then
/// turned `angle` radians clockwise about `center`. `longer` is the
/// picture's longer side.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn placed(
  center: AnnotationPoint,
  angle: f64,
  tilt: ImageTilt,
  longer: f64,
  local: AnnotationPoint,
) -> AnnotationPoint {
  from_local(center, angle, tilt.tilted(longer, local))
}

/// The picture's own point drawn at `point`, or `None` where nothing of the
/// picture is drawn there.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn unplaced(
  center: AnnotationPoint,
  angle: f64,
  tilt: ImageTilt,
  longer: f64,
  point: AnnotationPoint,
) -> Option<AnnotationPoint> {
  tilt.untilted(longer, to_local(center, angle, point))
}

/// Turns `shape`, where it is an image, to `rotation`: tilted in depth about
/// X and Y, and turned flat about Z, which runs counter-clockwise as the
/// image's own angle runs clockwise. Each angle is held to what a turn can
/// reach, and an image tilted neither way is flat again. The twin of
/// `withImageRotation` in `src/features/editor/annotations/image-rotation.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn turn_to(shape: &mut crate::AnnotationShape, rotation: SceneRotation) {
  let crate::AnnotationShape::Image { angle, tilt, .. } = shape else {
    return;
  };
  if ![rotation.x, rotation.y, rotation.z]
    .iter()
    .all(|angle| angle.is_finite())
  {
    return;
  }
  let rotation = rotation.held();
  *angle = -rotation.z.to_radians();
  *tilt = Some(ImageTilt {
    x: rotation.x,
    y: rotation.y,
  })
  .filter(|tilt| !tilt.is_flat());
}
