// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use super::{
  detector::{GlideDetection, GlideDetector, GlideSample},
  folds::GlideDetectorOptions,
  regions::GlideRegion,
  settling::GlidePhase,
  stroke::PUSH_GAP_MS,
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

  /// Lets time pass; answers whether the next step is ready.
  fn settle(&mut self) -> bool {
    self.detector.settle(self.now).phase == GlidePhase::Ready
  }

  /// The hand stops long enough to end the push.
  fn stop(&mut self) -> bool {
    self.advance(PUSH_GAP_MS);
    self.settle()
  }

  fn flick(&mut self, stroke: Stroke) -> GlideDetection {
    self.stop();
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
