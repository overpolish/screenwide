// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The surface a recording's erased box follows across its clip, read from
//! the ring round the box every `SAMPLE_MS`, and the export's held fills.

use crate::editor::annotations::redact::held::HeldFill;
use crate::editor::annotations::redact::native::{
  draw_points, held_fill, painted_bounds, RedactPicture,
};
#[cfg(target_os = "windows")]
use crate::editor::annotations::redact::surface_timeline::surface_at;
use crate::editor::annotations::redact::surface_timeline::{timeline, SAMPLE_MS};
use crate::editor::annotations::timing::RecordingAnnotationClip;
use crate::editor::annotations::{AnnotationRedaction, AnnotationShape};
use crate::editor::surface_colour::surrounding_colour;
use crate::screenshots::CapturedImage;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Whether a redaction reads the picture: erase for its surface, and a
/// secure pixelation for its zones and the surface its blank blocks show.
pub(crate) fn reads_picture(redaction: AnnotationRedaction) -> bool {
  matches!(
    redaction,
    AnnotationRedaction::Erase | AnnotationRedaction::Pixelate
  )
}

/// The surface timeline of `clip`'s box across the clip, from one decode
/// pass over the recording at `path`. Empty where it could not be read; cut
/// short, and to be thrown away, once `stop` is raised.
pub(crate) fn clip_surfaces(
  path: &std::path::Path,
  duration_ms: u64,
  clip: &RecordingAnnotationClip,
  stop: &AtomicBool,
) -> Vec<[f32; 2]> {
  let AnnotationShape::Redact { start, end, .. } = clip.annotation.shape else {
    return Vec::new();
  };
  let times: Vec<u64> = (clip.start_ms..clip.end_ms)
    .step_by(SAMPLE_MS as usize)
    .collect();
  let mut samples = Vec::with_capacity(times.len());
  let read = super::platform::each_source_frame(path, &times, duration_ms, |time, frame| {
    let bounds = painted_bounds(draw_points(start, end), frame.width, frame.height);
    let colour = surrounding_colour(&frame.rgba, frame.width, frame.height, bounds);
    samples.push((time - clip.start_ms, colour));
    !stop.load(Ordering::Relaxed)
  });
  if read.is_err() {
    return Vec::new();
  }
  timeline(&samples)
}

/// Hands each redaction among an export's `clips` that reads the picture its
/// fill: a secure pixelation's zones from its clip's first frame, and the
/// surface timeline across the clip. Each first frame is decoded once,
/// however many clips start on it. `capture_width_points` is how many
/// logical points the recording is wide.
pub(crate) fn attach_for_export(
  path: &std::path::Path,
  duration_ms: u64,
  clips: &mut [RecordingAnnotationClip],
  capture_width_points: f64,
) {
  let mut frames: HashMap<u64, Option<CapturedImage>> = HashMap::new();
  for clip in clips {
    let AnnotationShape::Redact { start, end, .. } = clip.annotation.shape else {
      continue;
    };
    if !reads_picture(clip.annotation.style.redaction) {
      continue;
    }
    let frame = frames.entry(clip.start_ms).or_insert_with(|| {
      super::platform::source_frame_image(path, clip.start_ms, duration_ms).ok()
    });
    // A frame that cannot be decoded leaves the box flat in its own colour,
    // which hides as much.
    let Some(frame) = frame else {
      continue;
    };
    let picture = RedactPicture::new(&frame.rgba, frame.width, frame.height, capture_width_points);
    let fill = HeldFill {
      surfaces: clip_surfaces(path, duration_ms, clip, &AtomicBool::new(false)),
      ..held_fill(start, end, &clip.annotation.style, &picture)
    };
    clip.annotation.held = Some(Arc::new(fill));
  }
}

/// Each of an export frame's redactions whose fill carries a surface
/// timeline, holding instead the surface that timeline gives `source_ms`
/// into the recording. `clips` are the export's clips, matched by id. The
/// Metal export resolves this in its own frame loop; the Windows one packs
/// each frame from Rust, so it resolves it here.
#[cfg(target_os = "windows")]
pub(crate) fn resolve_surfaces(
  annotations: &mut [crate::editor::annotations::Annotation],
  clips: &[RecordingAnnotationClip],
  source_ms: u64,
) {
  for annotation in annotations {
    let Some(held) = annotation
      .held
      .as_deref()
      .filter(|held| !held.surfaces.is_empty())
    else {
      continue;
    };
    let Some(clip) = clips
      .iter()
      .find(|clip| clip.annotation.id == annotation.id)
    else {
      continue;
    };
    let elapsed = source_ms.saturating_sub(clip.start_ms) as f32;
    let surface = surface_at(&held.surfaces, elapsed)
      .map(|surface| surface.map(|channel| (channel * 255.0).round() as u8));
    annotation.held = Some(Arc::new(HeldFill {
      surface: surface.or(held.surface),
      columns: held.columns,
      blocks: held.blocks,
      inks: held.inks.clone(),
      surfaces: Vec::new(),
    }));
  }
}
