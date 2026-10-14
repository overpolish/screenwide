// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The captions an export draws over each frame, timed on the edited video
//! its timeline lays out of the source.

use super::*;
use screenwide_captions::{CaptionMoment, CaptionTrack};
use screenwide_compositor::annotation_gpu::{caption_overlay, PreparedArrows};
use screenwide_timeline_edit::TimelinePlan;

pub(super) struct FrameCaptions<'a> {
  track: &'a CaptionTrack,
  timeline: Option<&'a TimelinePlan>,
  /// The canvas the captions are set on, in its pixels.
  canvas: (u32, u32),
}

impl<'a> FrameCaptions<'a> {
  /// The captions `request` draws on a canvas `canvas` in size, if any.
  pub(super) fn for_request(request: &CursorExportRequest<'a>, canvas: (u32, u32)) -> Option<Self> {
    Some(Self {
      track: request.captions?,
      timeline: request.timeline,
      canvas,
    })
  }

  /// Draws the captions showing at the frame `source_us` into the source,
  /// exposed over the `frame_ms` of output time it covers, over everything
  /// `prepared` draws.
  pub(super) fn draw_over(&self, prepared: &mut PreparedArrows, source_us: u64, frame_ms: f32) {
    let output_us = self
      .timeline
      .and_then(|plan| plan.source_to_output_us(source_us))
      .unwrap_or(source_us);
    let moment = CaptionMoment {
      at_ms: output_us as f64 / 1_000.0,
      exposure_ms: f64::from(frame_ms),
    };
    prepared.draw_above(caption_overlay(self.track, moment, self.canvas));
  }
}
