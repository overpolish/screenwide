// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the scenes make of one frame, placed on the timeline the clips are
//! timed on. A clip arrives over its first transition window and leaves over
//! its last, so a speed change never hurries a transition. Two clips that
//! butt hand over directly, without passing back through the composition
//! outside them; a clip that starts the video is in place from its first
//! frame, and one that runs on to the end never leaves.

use screenwide_annotations::timing::{scene_blend, video_ms, FrameTime};
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::model::{RecordingSceneClip, RecordingScenePreset};
use screenwide_scenes::motion::SceneMotion;
use screenwide_scenes::placement::Placement;
use screenwide_scenes::{CameraOverlaySettings, Measure};
use screenwide_timeline::{rate_at, TimelineRange};

use super::target::Base;

/// The placement the scenes give the frame `timeline_ms` into a video
/// `video_ms` long; `None` outside every clip and in a scene that stands idle.
/// Timed by the clock annotations move to their scene places by.
fn placement_at(
  clips: &[RecordingSceneClip],
  (timeline_ms, video_ms): (u64, Option<u64>),
  base: &Base,
) -> Option<Placement> {
  let blend = scene_blend(clips, (timeline_ms, video_ms), |clip| {
    base.target(clip).is_some()
  })?;
  let placed = |index: Option<usize>| {
    index
      .and_then(|index| base.target(&clips[index]))
      .unwrap_or(base.placement)
  };
  let to = placed(blend.to);
  if blend.from == blend.to {
    return Some(to);
  }
  Some(placed(blend.from).toward(to, blend.share))
}

/// How opaque the screen's annotations are drawn on the frame `timeline_ms`
/// into a video `video_ms` long, on the clock the panes move by: as the
/// screen is, except in a blank scene, which hides the screen but keeps what
/// is drawn on it over the background. Whole outside every clip.
fn annotations_opacity(
  clips: &[RecordingSceneClip],
  (timeline_ms, video_ms): (u64, Option<u64>),
  base: &Base,
) -> f64 {
  let shown = |index: Option<usize>| match index.map(|index| &clips[index]) {
    Some(clip) if clip.preset == RecordingScenePreset::Blank => 1.0,
    Some(clip) => base.target(clip).map_or(1.0, |placed| placed.opacity.0),
    None => 1.0,
  };
  scene_blend(clips, (timeline_ms, video_ms), |clip| {
    base.target(clip).is_some()
  })
  .map_or(1.0, |blend| {
    let (from, to) = (shown(blend.from), shown(blend.to));
    from + (to - from) * blend.share
  })
}

/// Arranges the screen's `output`, in the canvas's output pixels, and the
/// camera's overlay on the same canvas where `camera` gives one with the
/// camera's source size, as `clips` have them on the frame `source_ms` into
/// the recording, which `ranges` place on the timeline. Both are left as
/// they are outside every clip, which the answer says: `true` where a scene
/// placed them. `frame_ms` is how much recording time the frame covers:
/// where the panes moved over it, `output` carries that motion for the
/// compositor to blur; a still passes zero. The canvas's background is a
/// scene's own where it has one, changing on a clock of its own that starts
/// and ends a little past the clip's edges, whatever the answer.
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
  super::background::dress(clips, (timeline_ms, video_ms), &base, output);
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
  // A scene that pads its screen grows the crop outwards past its box, the
  // picture shown only in the box and the rest filled with the inset colour,
  // as the recording's own inset pads its crop.
  let inset = placed.inset.max(0.0);
  output.crop_x = placed.screen.x - inset;
  output.crop_y = placed.screen.y - inset;
  output.crop_width = placed.screen.width + inset * 2.0;
  output.crop_height = placed.screen.height + inset * 2.0;
  output.scene_window = (inset > 0.0).then_some([
    placed.screen.x,
    placed.screen.y,
    placed.screen.width,
    placed.screen.height,
  ]);
  // The placement holds the corners in canvas pixels, whatever the scene
  // measured them in; the layers draw them as shares of the canvas. A length
  // that already draws them is kept, so a pane the scene leaves alone keeps
  // the recording's own, a circle included.
  let canvas = output.canvas_shortest();
  let held = |own: Measure, frame: f64, pixels: f64| {
    if (own.radius_pixels(frame, canvas) - pixels).abs() < 1e-9 {
      own
    } else if canvas > 0.0 {
      Measure::Canvas(pixels / canvas * 100.0)
    } else {
      Measure::default()
    }
  };
  output.radius = held(
    output.radius,
    output.crop_width.min(output.crop_height),
    placed.radius.0,
  );
  output.scene_opacity = Some([
    placed.opacity.0 as f32,
    placed.opacity.1 as f32,
    annotations_opacity(clips, (timeline_ms, video_ms), &base) as f32,
  ]);
  output.scene_camera_front = Some(placed.camera_front as f32);
  // The scene's turn replaces the layer's own, upright where it gives none.
  output.rotation = Some(placed.rotation).filter(|rotation| !rotation.is_upright());
  if let Some((overlay, _)) = camera {
    overlay.frame_x = placed.frame.x;
    overlay.frame_y = placed.frame.y;
    overlay.frame_width = placed.frame.width;
    overlay.frame_height = placed.frame.height;
    (overlay.camera_x, overlay.camera_y, overlay.camera_width) = placed.camera;
    overlay.radius = held(
      overlay.radius,
      placed.frame.width.min(placed.frame.height),
      placed.radius.1,
    );
  }
  true
}
