// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The stroke in hand, and the annotation it describes.

use std::time::Instant;

use crate::editor::annotations::arrow::new_arrow;
use crate::editor::annotations::counter::new_counter;
use crate::editor::annotations::outline::model::new_shape;
use crate::editor::annotations::spotlight::model::{default_spotlight_style, new_spotlight};
use crate::editor::annotations::{Annotation, AnnotationKind, AnnotationPoint, AnnotationStyle};

/// The stroke in hand: the annotation it makes, when and where it started, and
/// where the pointer is now. The start time is what the annotation is timed
/// from, so a clip covers the drawing rather than beginning once it is over.
///
/// The shape, the number and the aim are taken at the press and held: a tool
/// picked up mid-drag chooses what the *next* annotation is, rather than
/// reshaping the one being drawn. The dress is read every frame, so a colour
/// changed mid-drag shows on the stroke in hand.
pub(super) struct Stroke {
  pub(super) id: String,
  pub(super) started_at: Instant,
  pub(super) start: AnnotationPoint,
  pub(super) end: AnnotationPoint,
  pub(super) shape: AnnotationKind,
  /// The counter's place in the order it was dropped in, and where its tail
  /// points. Neither is read for an arrow.
  pub(super) value: u32,
  pub(super) angle: f64,
  /// Whether a highlight is laid by hand over the box the drag spans rather
  /// than fitted to text: the tool's choice at the press.
  pub(super) manual: bool,
  /// A hand-drawn shape's wobble, held for the whole stroke so it does not
  /// wobble anew each frame. A highlight holds its own with its capture.
  pub(super) seed: u32,
}

impl Stroke {
  /// The dress the stroke is drawn in. An arrow's stroke and a counter's disc
  /// are different measurements of different things, so the width comes from
  /// the setting that belongs to the shape; a highlight has one marker, and a
  /// shape's pen is an arrow's. A highlight and a shape are each drawn by
  /// hand by their own choice, and a spotlight has its own corners, fade and
  /// blur. The overlay offers no text or redaction tool, and its settings
  /// refuse both, so neither ever reaches here.
  pub(super) fn style(&self) -> AnnotationStyle {
    let settings = super::super::settings::current();
    let shape = self.shape == AnnotationKind::Shape;
    if self.shape == AnnotationKind::Spotlight {
      return AnnotationStyle {
        blur: settings.spotlight_blur,
        radius: settings.spotlight_radius,
        softness: settings.spotlight_softness,
        ..default_spotlight_style()
      };
    }
    AnnotationStyle {
      align: Default::default(),
      blur: false,
      color: settings.default_color,
      head: settings.default_head,
      hand_drawn: if shape {
        settings.shape_hand_drawn
      } else {
        settings.highlight_hand_drawn
      },
      manual: self.manual,
      radius: if shape { settings.shape_radius } else { 0.0 },
      redaction: Default::default(),
      softness: 0.0,
      strength: 0.0,
      width: match self.shape {
        AnnotationKind::Arrow
        | AnnotationKind::Text
        | AnnotationKind::Redact
        | AnnotationKind::Shape
        | AnnotationKind::Spotlight => settings.default_width,
        AnnotationKind::Counter => settings.default_counter_size,
        AnnotationKind::Highlight => {
          crate::editor::annotations::highlight::model::NEW_HIGHLIGHT_WIDTH
        }
      },
    }
  }

  /// The annotation the stroke currently describes, built by the editor's own
  /// constructors so an annotation drawn live and one drawn in the editor are
  /// the same shape from the first frame.
  ///
  /// A counter sits where the pointer is rather than where the press landed:
  /// it is dropped whole, and the same drag carries it, exactly as a fresh
  /// counter in the editor is carried.
  pub(super) fn annotation(&self) -> Option<Annotation> {
    let style = self.style();
    match self.shape {
      AnnotationKind::Arrow => Some(new_arrow(
        self.id.clone(),
        self.start,
        self.end,
        Some(&style),
      )),
      AnnotationKind::Counter => Some(new_counter(
        self.id.clone(),
        self.end,
        self.value,
        Some(&style),
        Some(self.angle),
      )),
      AnnotationKind::Highlight => Some(super::super::highlight::annotation(
        &self.id, self.start, self.end, &style,
      )),
      AnnotationKind::Shape => Some(new_shape(
        self.id.clone(),
        [self.start, self.end],
        self.seed,
        Some(&style),
      )),
      AnnotationKind::Spotlight => Some(new_spotlight(
        self.id.clone(),
        [self.start, self.end],
        Some(&style),
      )),
      AnnotationKind::Text | AnnotationKind::Redact => None,
    }
  }

  /// Whether the stroke has anything to show. An arrow, a shape or a
  /// spotlight with both ends in one place is a blob, and the press that
  /// starts every stroke would flash one before the drag begins; a counter is
  /// an annotation the moment it is dropped. A highlight has nothing to
  /// recolour until the desktop under it is read.
  pub(super) fn is_drawn(&self) -> bool {
    match self.shape {
      AnnotationKind::Arrow | AnnotationKind::Shape | AnnotationKind::Spotlight => {
        self.start != self.end
      }
      AnnotationKind::Highlight => {
        self.start != self.end && super::super::highlight::ready(&self.id)
      }
      AnnotationKind::Counter => true,
      AnnotationKind::Text | AnnotationKind::Redact => false,
    }
  }
}
