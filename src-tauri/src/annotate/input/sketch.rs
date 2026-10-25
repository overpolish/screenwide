// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Strokes drawn apart from the live overlay, for a window that shows
//! Annotate at work: the same gestures making the same annotations, dressed
//! by the settings they are given and kept in a list of their own rather
//! than the session's.

use std::time::Instant;

use screenwide_annotations::{Annotation, AnnotationKind, AnnotationPoint};

use super::super::settings::AnnotateSettings;
use super::stroke::Stroke;
use super::{step, Tool};

#[derive(Default)]
pub(in crate::annotate) struct Sketch {
  drawing: Option<Stroke>,
  drawn: Vec<Annotation>,
}

impl Sketch {
  /// A pointer step in the sketch's points, with `shape` in hand: 0 down,
  /// 1 drag, 2 up.
  pub(in crate::annotate) fn pointer(
    &mut self,
    settings: &AnnotateSettings,
    shape: AnnotationKind,
    phase: u32,
    point: AnnotationPoint,
  ) {
    let tool = Tool {
      shape,
      value: screenwide_annotations::counter::next_counter_value(&self.drawn),
      angle: settings.default_counter_angle,
      manual: settings.highlight_manual,
      settings: settings.clone(),
    };
    if let Some((annotation, _)) = step(&mut self.drawing, phase, Instant::now(), point, &tool) {
      self.drawn.push(annotation);
    }
  }

  /// What is on screen, with the stroke in hand last.
  pub(in crate::annotate) fn annotations(&self, settings: &AnnotateSettings) -> Vec<Annotation> {
    let in_hand = self
      .drawing
      .as_ref()
      .filter(|stroke| stroke.is_drawn())
      .and_then(|stroke| stroke.annotation(settings));
    self.drawn.iter().cloned().chain(in_hand).collect()
  }

  pub(in crate::annotate) fn clear(&mut self) {
    self.drawing = None;
    self.drawn.clear();
  }
}
