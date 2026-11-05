// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When the scenes change: the one clock the picture's arrangement and an
//! annotation's place in each scene both move by, so an annotation glides to
//! where a scene puts it while the scene itself arrives.

use screenwide_scenes::RecordingSceneClip;
use screenwide_timeline::easing::ease_in_out_cubic;

use super::frame_time::{covers, progress, reaches_video_ends};

/// How long a scene takes to arrive or leave, in output milliseconds. A clip
/// too short for two of them gives each half of its length.
pub const SCENE_TRANSITION_MS: f32 = 600.0;

/// Where the scenes stand on one frame: on their way `from` one place `to`
/// another, `share` of the way there, eased. A place is the index of the
/// clip that sets it, or `None` for the composition outside every scene.
/// Settled in a clip, both are that clip and the share is one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneBlend {
  pub from: Option<usize>,
  pub to: Option<usize>,
  pub share: f64,
}

/// The scenes on the frame `timeline_ms` into a video `video_ms` long;
/// `None` outside every clip and in one that `places` says sets nothing,
/// which stands idle. A clip arrives over its first transition window, from
/// the clip butting up before it where that one places, and leaves over its
/// last for the composition outside, unless the clip butting up after it
/// places and takes over. A clip at either end of the video holds there:
/// there is nothing before it to arrive from, or after it to leave for, and
/// its other transition may take the whole clip.
pub fn scene_blend(
  clips: &[RecordingSceneClip],
  (timeline_ms, video_ms): (u64, Option<u64>),
  places: impl Fn(&RecordingSceneClip) -> bool,
) -> Option<SceneBlend> {
  let index = clips
    .iter()
    .position(|clip| covers(video_ms, [clip.start_ms, clip.end_ms], timeline_ms))?;
  let clip = &clips[index];
  if !places(clip) {
    return None;
  }
  let (elapsed, length) = progress([clip.start_ms, clip.end_ms], timeline_ms)?;
  let (starts_video, ends_video) = reaches_video_ends(video_ms, [clip.start_ms, clip.end_ms]);
  let window = SCENE_TRANSITION_MS.min(if starts_video || ends_video {
    length
  } else {
    length / 2.0
  });
  let ease = |share: f32| f64::from(ease_in_out_cubic(share.clamp(0.0, 1.0)));
  if !starts_video && elapsed < window {
    let from = index
      .checked_sub(1)
      .filter(|&previous| clips[previous].end_ms == clip.start_ms && places(&clips[previous]));
    return Some(SceneBlend {
      from,
      to: Some(index),
      share: ease(elapsed / window),
    });
  }
  let handed_on = clips
    .get(index + 1)
    .is_some_and(|next| next.start_ms == clip.end_ms && places(next));
  if !handed_on && !ends_video && length - elapsed < window {
    return Some(SceneBlend {
      from: None,
      to: Some(index),
      share: ease((length - elapsed) / window),
    });
  }
  Some(SceneBlend {
    from: Some(index),
    to: Some(index),
    share: 1.0,
  })
}

/// How much of the cursor shows on the frame `timeline_ms` into a video
/// `video_ms` long, from nothing to whole: `shown`, the recording's own
/// choice, outside every scene and in one that makes none of its own; a
/// scene's own choice inside it, fading in and out on the scene's clock as
/// the picture does.
pub fn cursor_in_scenes(
  clips: &[RecordingSceneClip],
  shown: bool,
  (timeline_ms, video_ms): (u64, Option<u64>),
) -> f32 {
  let shows = |index: Option<usize>| {
    let shows = index.and_then(|index| clips[index].cursor).unwrap_or(shown);
    if shows {
      1.0
    } else {
      0.0
    }
  };
  match scene_blend(clips, (timeline_ms, video_ms), |_| true) {
    Some(blend) => {
      let (from, to) = (shows(blend.from), shows(blend.to));
      from + (to - from) * blend.share as f32
    }
    None => shows(None),
  }
}

/// Whether the cursor shows anywhere in the video: `shown`, the recording's
/// own choice, or a scene that shows it over that.
pub fn cursor_shown_anywhere(clips: &[RecordingSceneClip], shown: bool) -> bool {
  shown || clips.iter().any(|clip| clip.cursor == Some(true))
}

#[cfg(test)]
mod tests;
