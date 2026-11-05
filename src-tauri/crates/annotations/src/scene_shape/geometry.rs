// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which of each kind's fields are its geometry, a scene's to change, and how
//! that geometry travels from one scene's place to another's.

use crate::{AnnotationPoint, AnnotationShape};

impl AnnotationShape {
  /// This shape with `geometry`'s geometry: what it says and is kept, where
  /// it sits taken from `geometry`. A shape of another kind leaves it whole.
  pub fn with_geometry_of(&self, geometry: &Self) -> Self {
    use AnnotationShape as S;
    let mut shape = self.clone();
    match (&mut shape, geometry) {
      (
        S::Arrow {
          start,
          control,
          end,
        },
        S::Arrow {
          start: s,
          control: c,
          end: e,
        },
      ) => (*start, *control, *end) = (*s, *c, *e),
      (
        S::Counter { center, angle, .. },
        S::Counter {
          center: c,
          angle: a,
          ..
        },
      ) => {
        (*center, *angle) = (*c, *a);
      }
      (
        S::Text {
          origin, controls, ..
        },
        S::Text {
          origin: o,
          controls: c,
          ..
        },
      ) => (*origin, *controls) = (*o, c.clone()),
      (
        S::Redact { start, end, .. },
        S::Redact {
          start: s, end: e, ..
        },
      )
      | (
        S::Shape { start, end, .. },
        S::Shape {
          start: s, end: e, ..
        },
      )
      | (S::Spotlight { start, end }, S::Spotlight { start: s, end: e })
      | (S::Focus { start, end }, S::Focus { start: s, end: e }) => (*start, *end) = (*s, *e),
      (
        S::Highlight {
          start, end, bands, ..
        },
        S::Highlight {
          start: s,
          end: e,
          bands: b,
          ..
        },
      ) => (*start, *end, *bands) = (*s, *e, b.clone()),
      (S::Draw { points, .. }, S::Draw { points: p, .. }) => *points = p.clone(),
      (
        S::Magnify { start, end, zoom },
        S::Magnify {
          start: s,
          end: e,
          zoom: z,
        },
      ) => (*start, *end, *zoom) = (*s, *e, *z),
      (
        S::Image {
          center,
          size,
          angle,
          tilt,
          flip,
          ..
        },
        S::Image {
          center: c,
          size: z,
          angle: a,
          tilt: t,
          flip: f,
          ..
        },
      ) => (*center, *size, *angle, *tilt, *flip) = (*c, *z, *a, *t, *f),
      _ => {}
    }
    shape
  }

  /// This shape `share` of the way to `to`'s geometry, eased by the caller:
  /// points, sizes, turns and zooms travel; what cannot travel, such as a
  /// stroke of another length or a mirror, changes halfway. Both are the
  /// same annotation, so only geometry differs.
  pub fn toward(&self, to: &Self, share: f64) -> Self {
    use AnnotationShape as S;
    let lerp = |a: f64, b: f64| a + (b - a) * share;
    let point = |a: AnnotationPoint, b: AnnotationPoint| AnnotationPoint {
      x: lerp(a.x, b.x),
      y: lerp(a.y, b.y),
    };
    let halfway = share >= 0.5;
    let mut shape = self.with_geometry_of(if halfway { to } else { self });
    match (&mut shape, self, to) {
      (
        S::Arrow {
          start,
          control,
          end,
        },
        S::Arrow {
          start: s0,
          control: c0,
          end: e0,
        },
        S::Arrow {
          start: s1,
          control: c1,
          end: e1,
        },
      ) => (*start, *control, *end) = (point(*s0, *s1), point(*c0, *c1), point(*e0, *e1)),
      (
        S::Counter { center, angle, .. },
        S::Counter {
          center: c0,
          angle: a0,
          ..
        },
        S::Counter {
          center: c1,
          angle: a1,
          ..
        },
      ) => {
        *center = point(*c0, *c1);
        // The tail turns the short way round.
        let turn =
          (a1 - a0 + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI;
        *angle = a0 + turn * share;
      }
      (
        S::Text {
          origin, controls, ..
        },
        S::Text {
          origin: o0,
          controls: k0,
          ..
        },
        S::Text {
          origin: o1,
          controls: k1,
          ..
        },
      ) => {
        *origin = point(*o0, *o1);
        for (id, moved) in controls.iter_mut() {
          if let (
            Some(crate::text::model::TextControlPoint::Picture(a)),
            Some(crate::text::model::TextControlPoint::Picture(b)),
          ) = (k0.get(id), k1.get(id))
          {
            *moved = crate::text::model::TextControlPoint::Picture(point(*a, *b));
          }
        }
      }
      (
        S::Redact { start, end, .. },
        S::Redact {
          start: s0, end: e0, ..
        },
        S::Redact {
          start: s1, end: e1, ..
        },
      )
      | (
        S::Shape { start, end, .. },
        S::Shape {
          start: s0, end: e0, ..
        },
        S::Shape {
          start: s1, end: e1, ..
        },
      )
      | (
        S::Spotlight { start, end },
        S::Spotlight { start: s0, end: e0 },
        S::Spotlight { start: s1, end: e1 },
      )
      | (
        S::Focus { start, end },
        S::Focus { start: s0, end: e0 },
        S::Focus { start: s1, end: e1 },
      ) => {
        (*start, *end) = (point(*s0, *s1), point(*e0, *e1));
      }
      (
        S::Highlight {
          start, end, bands, ..
        },
        S::Highlight {
          start: s0,
          end: e0,
          bands: b0,
          ..
        },
        S::Highlight {
          start: s1,
          end: e1,
          bands: b1,
          ..
        },
      ) => {
        (*start, *end) = (point(*s0, *s1), point(*e0, *e1));
        if b0.len() == b1.len() {
          for (band, (a, b)) in bands.iter_mut().zip(b0.iter().zip(b1)) {
            band.left = lerp(a.left, b.left);
            band.top = lerp(a.top, b.top);
            band.right = lerp(a.right, b.right);
            band.bottom = lerp(a.bottom, b.bottom);
          }
        }
      }
      (S::Draw { points, .. }, S::Draw { points: p0, .. }, S::Draw { points: p1, .. }) => {
        if p0.len() == p1.len() {
          for (moved, (a, b)) in points.iter_mut().zip(p0.iter().zip(p1)) {
            *moved = point(*a, *b);
          }
        }
      }
      (
        S::Magnify { start, end, zoom },
        S::Magnify {
          start: s0,
          end: e0,
          zoom: z0,
        },
        S::Magnify {
          start: s1,
          end: e1,
          zoom: z1,
        },
      ) => (*start, *end, *zoom) = (point(*s0, *s1), point(*e0, *e1), lerp(*z0, *z1)),
      (
        S::Image {
          center,
          size,
          angle,
          tilt,
          ..
        },
        S::Image {
          center: c0,
          size: z0,
          angle: a0,
          tilt: t0,
          ..
        },
        S::Image {
          center: c1,
          size: z1,
          angle: a1,
          tilt: t1,
          ..
        },
      ) => {
        (*center, *size, *angle) = (point(*c0, *c1), lerp(*z0, *z1), lerp(*a0, *a1));
        let flat = crate::image::ImageTilt { x: 0.0, y: 0.0 };
        if t0.is_some() || t1.is_some() {
          let (a, b) = (t0.unwrap_or(flat), t1.unwrap_or(flat));
          *tilt = Some(crate::image::ImageTilt {
            x: lerp(a.x, b.x),
            y: lerp(a.y, b.y),
          })
          .filter(|tilt| !tilt.is_flat());
        }
      }
      _ => {}
    }
    shape
  }
}
