// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the scenes make of one frame, placed on the timeline the clips are
//! timed on. A clip arrives over its first transition window and leaves over
//! its last, so a speed change never hurries a transition. Two clips that
//! butt hand over directly, without passing back through the composition
//! outside them; a clip that starts the video is in place from its first
//! frame, and one that runs on to the end never leaves.

use screenwide_annotations::timing::{covers, progress, reaches_video_ends, video_ms, FrameTime};
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::framing::SceneFraming;
use screenwide_scenes::geometry::Rect;
use screenwide_scenes::model::RecordingSceneClip;
use screenwide_scenes::motion::SceneMotion;
use screenwide_scenes::placement::Placement;
use screenwide_scenes::CameraOverlaySettings;
use screenwide_timeline::easing::ease_in_out_cubic;
use screenwide_timeline::{rate_at, TimelineRange};

use super::target::Base;

/// How long a scene takes to arrive or leave, in output milliseconds. A clip
/// too short for two of them gives each half of its length.
const TRANSITION_MS: f32 = 600.0;

/// The placement the scenes give the frame `timeline_ms` into a video
/// `video_ms` long; `None` outside every clip and in a scene that stands idle.
fn placement_at(
  clips: &[RecordingSceneClip],
  (timeline_ms, video_ms): (u64, Option<u64>),
  base: &Base,
) -> Option<Placement> {
  let index = clips
    .iter()
    .position(|clip| covers(video_ms, [clip.start_ms, clip.end_ms], timeline_ms))?;
  let clip = &clips[index];
  let target = base.target(clip)?;
  let (elapsed, length) = progress([clip.start_ms, clip.end_ms], timeline_ms)?;
  // A scene at either end of the video holds there: there is nothing before
  // it to arrive from, or after it to leave for. Its other transition may then
  // take the whole clip.
  let (starts_video, ends_video) = reaches_video_ends(video_ms, [clip.start_ms, clip.end_ms]);
  let window = TRANSITION_MS.min(if starts_video || ends_video {
    length
  } else {
    length / 2.0
  });
  let ease = |share: f32| f64::from(ease_in_out_cubic(share.clamp(0.0, 1.0)));
  if !starts_video && elapsed < window {
    let from = index
      .checked_sub(1)
      .map(|previous| &clips[previous])
      .filter(|previous| previous.end_ms == clip.start_ms)
      .and_then(|previous| base.target(previous))
      .unwrap_or(base.placement);
    return Some(from.toward(target, ease(elapsed / window)));
  }
  let handed_on = clips
    .get(index + 1)
    .is_some_and(|next| next.start_ms == clip.end_ms && base.target(next).is_some());
  if !handed_on && !ends_video && length - elapsed < window {
    return Some(
      base
        .placement
        .toward(target, ease((length - elapsed) / window)),
    );
  }
  Some(target)
}

/// Arranges the screen's `output`, in the canvas's output pixels, and the
/// camera's overlay on the same canvas where `camera` gives one with the
/// camera's source size, as `clips` have them on the frame `source_ms` into
/// the recording, which `ranges` place on the timeline. Both are left as
/// they are outside every clip, which the answer says: `true` where a scene
/// placed them. `frame_ms` is how much recording time the frame covers:
/// where the panes moved over it, `output` carries that motion for the
/// compositor to blur; a still passes zero.
pub fn arrange(
  clips: &[RecordingSceneClip],
  ranges: &[TimelineRange],
  source_ms: u64,
  frame_ms: f32,
  output: &mut ScreenshotOutputSettings,
  camera: Option<(&mut CameraOverlaySettings, (u32, u32))>,
) -> bool {
  arrange_following(clips, ranges, (source_ms, frame_ms), output, camera, None)
}

/// [`arrange`], where the camera follows the face at `face`: where the face
/// is on this frame, as shares of the camera picture. A clip that follows it
/// centres it in the clip's box, and where the recording's camera follows
/// it, the camera outside every scene is moved to follow it too, though no
/// scene placed it. Without `face`, as before the face is found, the camera
/// is framed as it would be without following.
pub fn arrange_following(
  clips: &[RecordingSceneClip],
  ranges: &[TimelineRange],
  (source_ms, frame_ms): (u64, f32),
  output: &mut ScreenshotOutputSettings,
  camera: Option<(&mut CameraOverlaySettings, (u32, u32))>,
  face: Option<(f64, f64)>,
) -> bool {
  let camera = camera.filter(|(_, (width, height))| *width > 0 && *height > 0);
  let base = Base::of(
    output,
    camera
      .as_ref()
      .map(|(overlay, (width, height))| (&**overlay, f64::from(*width) / f64::from(*height))),
    face,
  );
  let timeline_ms = FrameTime::on(ranges, source_ms).timeline_ms;
  let video_ms = video_ms(ranges);
  let placed = (!clips.is_empty() && output.has_placement())
    .then(|| placement_at(clips, (timeline_ms, video_ms), &base))
    .flatten();
  let Some(placed) = placed else {
    if let Some((overlay, _)) = camera {
      (overlay.camera_x, overlay.camera_y, overlay.camera_width) = base.placement.camera;
    }
    return false;
  };
  if frame_ms > 0.0 {
    // Opened a frame's length earlier on the timeline; before the first clip, outside them.
    let frame_on_timeline = frame_ms / rate_at(ranges, source_ms as f64 * 1_000.0) as f32;
    let opened_ms = timeline_ms.saturating_sub(frame_on_timeline.round() as u64);
    let opened = placement_at(clips, (opened_ms, video_ms), &base).unwrap_or(base.placement);
    output.scene_motion = SceneMotion::between(
      &opened,
      &placed,
      base.canvas,
      base.camera_aspect.unwrap_or(1.0),
    );
  }
  // Annotations belong to the screen's picture, so a scene grows and shrinks
  // them with it: their sizes are points of the capture, drawn this many more
  // canvas pixels to the point. The picture's width in points stays put, so
  // they keep their place and size on what they point at.
  output.capture_scale = output.size_scale() * placed.image.2 / base.placement.image.2;
  (output.image_x, output.image_y, output.image_width) = placed.image;
  output.crop_x = placed.screen.x;
  output.crop_y = placed.screen.y;
  output.crop_width = placed.screen.width;
  output.crop_height = placed.screen.height;
  output.radius_percent = placed.radius.0;
  output.scene_opacity = Some([placed.opacity.0 as f32, placed.opacity.1 as f32]);
  output.scene_camera_front = Some(placed.camera_front as f32);
  // The scene's turn replaces the layer's own, upright where it gives none.
  output.rotation = Some(placed.rotation).filter(|rotation| !rotation.is_upright());
  if let Some((overlay, _)) = camera {
    overlay.frame_x = placed.frame.x;
    overlay.frame_y = placed.frame.y;
    overlay.frame_width = placed.frame.width;
    overlay.frame_height = placed.frame.height;
    (overlay.camera_x, overlay.camera_y, overlay.camera_width) = placed.camera;
    overlay.radius_percent = placed.radius.1;
  }
  true
}

/// What a drag on the camera reframes `timeline_ms` into the edited video:
/// the index of the clip it falls in, the box that clip settles the camera
/// in, and the framing it shows there. `None` outside every clip, in a scene
/// that stands idle, and in one that hides the camera.
pub fn camera_target(
  clips: &[RecordingSceneClip],
  timeline_ms: u64,
  output: &ScreenshotOutputSettings,
  overlay: &CameraOverlaySettings,
  camera: (u32, u32),
) -> Option<(usize, Rect, SceneFraming)> {
  if camera.0 == 0 || camera.1 == 0 || !output.has_placement() {
    return None;
  }
  let aspect = f64::from(camera.0) / f64::from(camera.1);
  let base = Base::of(output, Some((overlay, aspect)), None);
  let index = clips
    .iter()
    .position(|clip| covers(None, [clip.start_ms, clip.end_ms], timeline_ms))?;
  let clip = &clips[index];
  let target = base.target(clip)?;
  if target.opacity.1 == 0.0 {
    return None;
  }
  let frame = target.frame;
  let framing = clip.camera.unwrap_or_else(|| base.unframed(clip, overlay));
  Some((index, frame, framing.within(frame, aspect)))
}
