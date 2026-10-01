// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  detector::{GlideDetection, GlideDetector, GlideSample},
  folds::GlideDetectorOptions,
  regions::GlideRegion,
};

#[derive(Clone, Copy, Default)]
struct Stroke {
  delta_x: f64,
  delta_y: f64,
  thirds: bool,
}

struct Gesture {
  detector: GlideDetector,
  now: f64,
}

impl Gesture {
  fn new() -> Self {
    Self::with_options(GlideDetectorOptions {
      opening_grace_ms: 0.0,
      ..GlideDetectorOptions::default()
    })
  }

  fn with_options(options: GlideDetectorOptions) -> Self {
    Self {
      detector: GlideDetector::new(options),
      now: 0.0,
    }
  }

  fn move_by(&mut self, stroke: Stroke) -> GlideDetection {
    self.detector.update(GlideSample {
      delta_x: stroke.delta_x,
      delta_y: stroke.delta_y,
      thirds: stroke.thirds,
      timestamp: self.now,
    })
  }

  fn advance(&mut self, milliseconds: f64) {
    self.now += milliseconds;
  }

  fn settle(&mut self) -> bool {
    self.detector.settle(self.now).became_ready
  }

  fn rest(&mut self) -> bool {
    self.advance(self.detector.options().rest_ms);
    self.settle()
  }

  fn flick(&mut self, stroke: Stroke) -> GlideDetection {
    self.rest();
    self.move_by(stroke)
  }
}

fn region(
  grid_cols: u32,
  col_start: u32,
  col_span: u32,
  row_start: u32,
  row_span: u32,
) -> GlideRegion {
  GlideRegion {
    grid_cols,
    col_start,
    col_span,
    row_start,
    row_span,
  }
}

fn stroke(delta_x: f64, delta_y: f64) -> Stroke {
  Stroke {
    delta_x,
    delta_y,
    thirds: false,
  }
}

mod detector_tests;
mod geometry_tests;
mod lifecycle_tests;
mod regions_tests;
mod runtime_tests;

mod intent_tests;
