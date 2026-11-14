// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When a scene's background takes over. The background keeps a clock of its
//! own beside the panes': each change starts a little ahead of their move and
//! runs a little longer, so the new background is already opening as the
//! screen starts to move and settles just after it lands.

use screenwide_scenes::RecordingSceneClip;
use screenwide_timeline::easing::ease_in_out_cubic;

use super::super::frame_time::{covers, reaches_video_ends};
use super::pane_window;

/// How long one background takes to bloom in over another, in output
/// milliseconds. A change that the next one would interrupt is hurried to
/// finish as the next begins.
pub const BACKGROUND_TRANSITION_MS: f64 = 800.0;

/// How far ahead of the panes' move a background starts to change, in
/// output milliseconds.
pub const BACKGROUND_LEAD_MS: f64 = 100.0;

/// The backgrounds on one frame: on their way `from` one place's `to`
/// another's, `share` of the way, eased. A place is the index of the clip
/// whose background it is, or `None` for the recording's own. Settled, both
/// are the same place and the share is one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackgroundBlend {
  pub from: Option<usize>,
  pub to: Option<usize>,
  pub share: f64,
}

impl BackgroundBlend {
  fn settled(place: Option<usize>) -> Self {
    Self {
      from: place,
      to: place,
      share: 1.0,
    }
  }
}

/// One change of background, starting `start_ms` into the video.
#[derive(Clone, Copy)]
struct Change {
  start_ms: f64,
  from: Option<usize>,
  to: Option<usize>,
}

/// Every change the clips make, in the order they start: each clip's
/// arrival, from the clip butting up before it or the recording, unless it
/// starts the video; and its leaving for the recording, unless the clip
/// butting up after it takes over or it ends the video. Each starts the lead
/// ahead of the panes' own move.
fn changes(
  clips: &[RecordingSceneClip],
  video_ms: Option<u64>,
) -> impl Iterator<Item = Change> + '_ {
  clips.iter().enumerate().flat_map(move |(index, clip)| {
    let (starts_video, ends_video) = reaches_video_ends(video_ms, [clip.start_ms, clip.end_ms]);
    let arrival = (!starts_video).then(|| Change {
      start_ms: clip.start_ms as f64 - BACKGROUND_LEAD_MS,
      from: index
        .checked_sub(1)
        .filter(|&previous| clips[previous].end_ms == clip.start_ms),
      to: Some(index),
    });
    let handed_on = clips
      .get(index + 1)
      .is_some_and(|next| next.start_ms == clip.end_ms);
    let length = clip.end_ms.saturating_sub(clip.start_ms) as f32;
    let leaving = (!handed_on && !ends_video).then(|| Change {
      start_ms: clip.end_ms as f64
        - f64::from(pane_window(length, starts_video))
        - BACKGROUND_LEAD_MS,
      from: Some(index),
      to: None,
    });
    arrival.into_iter().chain(leaving)
  })
}

/// The backgrounds on the frame `timeline_ms` into a video `video_ms` long.
/// `differs` says whether two places show different backgrounds: a change
/// between two that look alike is no change at all, so it neither blooms
/// nor hurries the change before it.
pub fn background_blend(
  clips: &[RecordingSceneClip],
  (timeline_ms, video_ms): (u64, Option<u64>),
  differs: impl Fn(Option<usize>, Option<usize>) -> bool,
) -> BackgroundBlend {
  let now = timeline_ms as f64;
  let mut current: Option<Change> = None;
  let mut upcoming = None;
  for change in changes(clips, video_ms).filter(|change| differs(change.from, change.to)) {
    if change.start_ms > now {
      upcoming = Some(change);
      break;
    }
    current = Some(change);
  }
  let Some(change) = current else {
    // Before the first change the background is the one it starts from;
    // with no change at all, whichever place the frame is in.
    return BackgroundBlend::settled(upcoming.map_or_else(
      || {
        clips
          .iter()
          .position(|clip| covers(video_ms, [clip.start_ms, clip.end_ms], timeline_ms))
      },
      |change| change.from,
    ));
  };
  let length = upcoming.map_or(BACKGROUND_TRANSITION_MS, |next| {
    BACKGROUND_TRANSITION_MS.min(next.start_ms - change.start_ms)
  });
  let elapsed = now - change.start_ms;
  if length <= 0.0 || elapsed >= length {
    return BackgroundBlend::settled(change.to);
  }
  BackgroundBlend {
    from: change.from,
    to: change.to,
    share: f64::from(ease_in_out_cubic((elapsed / length) as f32)),
  }
}

#[cfg(test)]
#[path = "background_tests.rs"]
mod tests;
