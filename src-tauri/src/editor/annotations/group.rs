// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Several annotations chosen together: the box drawn round them, and the
//! move that carries them all at once.
//!
//! A group is carried by one distance on screen. Its members may sit on
//! pictures drawn at different scales - a recording's screen and camera - so
//! each member carries its own factor from the pressed picture's source pixels
//! into its own. The members hidden at the playhead move with the rest: the
//! box round the group encloses them too, so the hand can see they will.

use super::snap::{SnapBox, SnapModifiers, SnapOffset, SnapRequest, SnapResult};
use super::{Annotation, AnnotationPoint, AnnotationShape};

#[cfg(test)]
mod tests;

/// The box round everything an annotation draws, in its source's pixels: the
/// box it aligns by where it has one, the points it is placed by, a
/// magnifier's loupe, and an arrow's stroke and heads either side of its line.
/// `source_per_size` turns a style's size into source pixels.
pub(crate) fn annotation_bounds(annotation: &Annotation, source_per_size: f64) -> Option<SnapBox> {
  let points = annotation.shape.points();
  let mut bounds = points
    .iter()
    .skip(1)
    .fold(point_box(points[0]), |bounds, point| {
      union(bounds, point_box(*point))
    });
  if let Some(field) = annotation
    .shape
    .field_box(annotation.style.width, source_per_size)
  {
    bounds = union(bounds, field);
  }
  match &annotation.shape {
    AnnotationShape::Arrow { .. } => {
      let reach = annotation.style.width.max(0.0) * source_per_size.max(0.0);
      bounds = SnapBox {
        x: bounds.x - reach,
        y: bounds.y - reach,
        width: bounds.width + reach * 2.0,
        height: bounds.height + reach * 2.0,
      };
    }
    AnnotationShape::Magnify {
      start,
      end,
      loupe,
      size,
    } => {
      // The loupe is the zoom area enlarged until its longer side is `size`.
      let longest = super::magnify::model::longest(*start, *end);
      if longest > 0.0 && *size > 0.0 {
        let width = (end.x - start.x).abs() * size / longest;
        let height = (end.y - start.y).abs() * size / longest;
        bounds = union(
          bounds,
          SnapBox {
            x: loupe.x - width / 2.0,
            y: loupe.y - height / 2.0,
            width,
            height,
          },
        );
      }
    }
    _ => {}
  }
  [bounds.x, bounds.y, bounds.width, bounds.height]
    .iter()
    .all(|value| value.is_finite())
    .then_some(bounds)
}

/// The box round every one of `annotations`, or `None` for none at all.
pub(crate) fn group_bounds<'a>(
  annotations: impl IntoIterator<Item = &'a Annotation>,
  source_per_size: f64,
) -> Option<SnapBox> {
  annotations
    .into_iter()
    .filter_map(|annotation| annotation_bounds(annotation, source_per_size))
    .reduce(union)
}

fn point_box(point: AnnotationPoint) -> SnapBox {
  SnapBox {
    x: point.x,
    y: point.y,
    width: 0.0,
    height: 0.0,
  }
}

fn union(a: SnapBox, b: SnapBox) -> SnapBox {
  let x = a.x.min(b.x);
  let y = a.y.min(b.y);
  SnapBox {
    x,
    y,
    width: (a.x + a.width).max(b.x + b.width) - x,
    height: (a.y + a.height).max(b.y + b.height) - y,
  }
}

/// `annotation` carried `offset` of its source's pixels.
pub(crate) fn carried(annotation: &Annotation, offset: SnapOffset) -> Annotation {
  let mut moved = annotation.clone();
  moved.shape = annotation.shape.mapped(|point| offset.apply(point));
  moved
}

/// One member of a group being carried: as it was when the press landed, and
/// how many of its own source pixels one source pixel of the pressed picture
/// spans on screen.
#[derive(Clone, Debug)]
pub(crate) struct GroupMember {
  pub(crate) before: Annotation,
  pub(crate) scale: f64,
}

/// A group carried by its body, measured from the press so a drag nudged back
/// and forth lands exactly where the pointer is.
#[derive(Clone, Debug)]
pub(crate) struct GroupMove {
  origin: AnnotationPoint,
  members: Vec<GroupMember>,
  /// The box round the members on the pressed picture: what snaps, as one.
  held: Option<SnapBox>,
}

impl GroupMove {
  pub(crate) fn new(
    origin: AnnotationPoint,
    members: Vec<GroupMember>,
    held: Option<SnapBox>,
  ) -> Self {
    Self {
      origin,
      members,
      held,
    }
  }

  /// Whether `id` is one of the members.
  pub(crate) fn holds(&self, id: &str) -> bool {
    self.members.iter().any(|member| member.before.id == id)
  }

  /// Every member carried as far as the pointer has come from the press, and
  /// what the carry snapped to. Shift holds the move to whichever axis it has
  /// come further along; the positional modifier lands the pressed picture's
  /// box on the guides and gaps the field offers.
  pub(crate) fn update(
    &self,
    point: AnnotationPoint,
    modifiers: SnapModifiers,
    snap: Option<SnapRequest<'_>>,
  ) -> (Vec<Annotation>, SnapResult) {
    let mut offset = SnapOffset {
      x: point.x - self.origin.x,
      y: point.y - self.origin.y,
    };
    if modifiers.shift {
      if offset.x.abs() >= offset.y.abs() {
        offset.y = 0.0;
      } else {
        offset.x = 0.0;
      }
    }
    let mut result = SnapResult::default();
    if let (true, Some(request), Some(held)) = (modifiers.position, snap, self.held) {
      let (snapped, landed) = request.boxed(held.moved(offset), None);
      offset.x += snapped.x;
      offset.y += snapped.y;
      result = landed;
    }
    let moved = self
      .members
      .iter()
      .map(|member| {
        carried(
          &member.before,
          SnapOffset {
            x: offset.x * member.scale,
            y: offset.y * member.scale,
          },
        )
      })
      .collect();
    (moved, result)
  }
}

/// One box the macOS chrome draws round a group, normalised over its layer's
/// source image, matching the C `ScreenwideAnnotationGroupBox`: the box round
/// one member, or round the whole group on that layer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativeAnnotationGroupBox {
  pub(crate) left: f64,
  pub(crate) top: f64,
  pub(crate) right: f64,
  pub(crate) bottom: f64,
  pub(crate) layer_id: i32,
  /// [`GROUP_BOX_MEMBER`] or [`GROUP_BOX_GROUP`].
  pub(crate) kind: u32,
}

const _: () = assert!(std::mem::size_of::<NativeAnnotationGroupBox>() == 40);

/// The box round one member shown at the playhead.
pub(crate) const GROUP_BOX_MEMBER: u32 = 0;
/// The box round every member on a layer, shown or not.
pub(crate) const GROUP_BOX_GROUP: u32 = 1;

/// `bounds` as the chrome places it: normalised over `source`.
pub(crate) fn native_group_box(
  bounds: SnapBox,
  source: (u32, u32),
  layer_id: i32,
  kind: u32,
) -> NativeAnnotationGroupBox {
  let across = f64::from(source.0.max(1));
  let down = f64::from(source.1.max(1));
  NativeAnnotationGroupBox {
    left: bounds.x / across,
    top: bounds.y / down,
    right: (bounds.x + bounds.width) / across,
    bottom: (bounds.y + bounds.height) / down,
    layer_id,
    kind,
  }
}

/// The boxes the chrome draws for the members of a group on one layer: one
/// round each member `shown` at the playhead, and one round all of `members`,
/// shown or not. Nothing for a layer with no member.
pub(crate) fn group_boxes(
  members: &[&Annotation],
  shown: impl Fn(&str) -> bool,
  source: (u32, u32),
  source_per_size: f64,
  layer_id: i32,
) -> Vec<NativeAnnotationGroupBox> {
  let Some(whole) = group_bounds(members.iter().copied(), source_per_size) else {
    return Vec::new();
  };
  members
    .iter()
    .filter(|member| shown(&member.id))
    .filter_map(|member| annotation_bounds(member, source_per_size))
    .map(|bounds| native_group_box(bounds, source, layer_id, GROUP_BOX_MEMBER))
    .chain(std::iter::once(native_group_box(
      whole,
      source,
      layer_id,
      GROUP_BOX_GROUP,
    )))
    .collect()
}
