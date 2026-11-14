// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The background a frame is drawn on: the recording's own, a scene's, or
//! one blooming in over the other on the backgrounds' own clock. The bloom
//! spreads from where the incoming screen lands, and where the screen zooms
//! both backgrounds are pushed the same way, as one camera would move them.

use screenwide_annotations::timing::background_blend;
use screenwide_output::{BackgroundTurn, CanvasBackground, ScreenshotOutputSettings};
use screenwide_scenes::model::RecordingSceneClip;

use super::target::Base;

/// How much larger the backgrounds are drawn by the end of a change that
/// zooms the screen far enough, as a share.
const PUSH: f64 = 0.06;

/// The zoom, as a ratio of the screen picture's widths, from which the push
/// is whole; a smaller zoom pushes less, and a move alone not at all.
const WHOLE_PUSH_ZOOM: f64 = 1.1;

/// The push for a change that zooms the screen picture by `zoom`.
fn push(zoom: f64) -> f64 {
  if !zoom.is_finite() || zoom <= 0.0 {
    return 0.0;
  }
  let steps = zoom.ln() / WHOLE_PUSH_ZOOM.ln();
  PUSH * steps.clamp(-1.0, 1.0)
}

/// Draws `output` on the background the frame `timeline_ms` into a video
/// `video_ms` long has: the scene's own where the frame is in one that has
/// one, and while one gives way to another, the incoming one in the
/// canvas's own fields with the outgoing one in its turn.
pub(super) fn dress(
  clips: &[RecordingSceneClip],
  (timeline_ms, video_ms): (u64, Option<u64>),
  base: &Base,
  output: &mut ScreenshotOutputSettings,
) {
  if clips.iter().all(|clip| clip.background.is_none()) {
    return;
  }
  let recording = output.background();
  let own = |place: Option<usize>| {
    place
      .and_then(|index| clips[index].background.as_ref())
      .unwrap_or(&recording)
  };
  let blend = background_blend(clips, (timeline_ms, video_ms), |from, to| {
    own(from) != own(to)
  });
  if blend.from != blend.to && blend.share < 1.0 {
    let placed = |place: Option<usize>| {
      place
        .and_then(|index| base.target(&clips[index]))
        .unwrap_or(base.placement)
    };
    let (from, to) = (placed(blend.from), placed(blend.to));
    output.background_turn = Some(BackgroundTurn {
      outgoing: CanvasBackground::worn_on(output, own(blend.from)),
      share: blend.share,
      centre: (
        (to.screen.x + to.screen.width / 2.0) / base.canvas.0.max(1.0),
        (to.screen.y + to.screen.height / 2.0) / base.canvas.1.max(1.0),
      ),
      push: push(to.image.2 / from.image.2),
    });
  }
  let incoming = own(blend.to);
  if incoming != &recording {
    output.wear_background(incoming);
  }
}
