// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the scenes make of one frame. A clip arrives over its first
//! transition window and leaves over its last, both measured in output time
//! over what the timeline keeps, so a speed change never hurries a transition.
//! Two clips that butt hand over directly, morphing from one to the other
//! without passing back through the composition the recording has outside
//! them, a clip that starts the video is in place from its first frame, and
//! one that runs on to the end of the video never leaves.

use super::framing::{drawn_frame, SceneFraming};
use super::geometry::{preset_panes, Rect};
use super::model::{RecordingSceneClip, RecordingScenePreset};
use super::motion::SceneMotion;
use super::placement::Placement;
use crate::editor::annotations::timing::{covers, output_progress, reaches_video_ends};
use crate::editor::effect_animation::ease_in_out_cubic;
use crate::editor::timeline_edit::TimelineRange;
use crate::editor::CameraOverlaySettings;
use crate::screenshots::ScreenshotOutputSettings;

/// How long a scene takes to arrive or leave, in output milliseconds. A clip
/// too short for two of them gives each half of its length.
const TRANSITION_MS: f32 = 600.0;

/// What a frame is arranged from: the composition outside every scene, the
/// canvas, and the camera picture's aspect where a camera is drawn into it.
struct Base {
  placement: Placement,
  canvas: (f64, f64),
  camera_aspect: Option<f64>,
}

impl Base {
  fn of(output: &ScreenshotOutputSettings, camera: Option<(&CameraOverlaySettings, f64)>) -> Self {
    let overlay = camera.map(|(overlay, _)| *overlay);
    // Where the camera is drawn rather than where its box is stored, so a
    // scene arriving starts from the camera on screen.
    let frame = camera.map_or(Rect::default(), |(overlay, aspect)| {
      drawn_frame(overlay, aspect)
    });
    Self {
      placement: Placement {
        screen: Rect {
          x: output.crop_x,
          y: output.crop_y,
          width: output.crop_width,
          height: output.crop_height,
        },
        image: (output.image_x, output.image_y, output.image_width),
        frame,
        camera: overlay.map_or((0.0, 0.0, 0.0), |overlay| {
          (overlay.camera_x, overlay.camera_y, overlay.camera_width)
        }),
        radius: (
          output.radius_percent,
          overlay.map_or(0.0, |overlay| overlay.radius_percent),
        ),
        opacity: (1.0, 1.0),
        camera_front: 1.0,
      },
      canvas: (f64::from(output.width), f64::from(output.height)),
      camera_aspect: camera.map(|(_, aspect)| aspect),
    }
  }

  /// Where `clip` settles the panes, or `None` for an arrangement that needs
  /// a camera where none is drawn into the picture: that scene stands idle.
  /// A pane the preset hides keeps the recording's own place, unseen.
  fn target(&self, clip: &RecordingSceneClip) -> Option<Placement> {
    let base = self.placement;
    let crop = base.screen;
    // Without a camera no preset that places one plays, so any shape will do.
    let preset = preset_panes(
      clip.preset,
      self.canvas,
      (
        crop.width / crop.height,
        self.camera_aspect.unwrap_or(16.0 / 9.0),
      ),
      clip.variant.unwrap_or_default(),
    );
    let (screen, frame, shown) = match (clip.boxes, preset) {
      (Some(boxes), _) => (
        boxes.screen.on(self.canvas),
        match boxes.camera {
          Some(camera) => {
            self.camera_aspect?;
            camera.on(self.canvas)
          }
          None => base.frame,
        },
        (true, true),
      ),
      (None, Some(panes)) => {
        if panes.camera.is_some() {
          self.camera_aspect?;
        }
        (
          panes.screen.unwrap_or(crop),
          panes.camera.unwrap_or(base.frame),
          (panes.screen.is_some(), panes.camera.is_some()),
        )
      }
      (None, None) => (
        crop,
        base.frame,
        (true, clip.preset != RecordingScenePreset::ScreenOnly),
      ),
    };
    // The screen's framing places the visible crop as if it were the
    // picture, so a zoom shows less of what the crop shows; the whole image
    // follows it.
    let (x, y, width) = clip
      .screen
      .unwrap_or(SceneFraming::WHOLE)
      .place(screen, crop.width / crop.height);
    let scale = width / crop.width;
    let height = width * crop.height / crop.width;
    let image = (
      x - width / 2.0 + (base.image.0 - crop.x) * scale,
      y - height / 2.0 + (base.image.1 - crop.y) * scale,
      base.image.2 * scale,
    );
    let camera = match (self.camera_aspect, clip.camera) {
      (Some(aspect), Some(framing)) => framing.place(frame, aspect),
      (Some(aspect), None) if clip.needs_camera() => SceneFraming::WHOLE.place(frame, aspect),
      _ => base.camera,
    };
    let radius = clip.radius.unwrap_or_default();
    // A camera filling the canvas is cut by the canvas's own corners, not
    // rounded again inside them.
    let camera_radius = if clip.boxes.is_none() && clip.preset == RecordingScenePreset::CameraOnly {
      0.0
    } else {
      base.radius.1
    };
    let opacity = |shown: bool| if shown { 1.0 } else { 0.0 };
    Some(Placement {
      screen,
      image,
      frame,
      camera,
      radius: (
        radius.screen.unwrap_or(base.radius.0),
        radius.camera.unwrap_or(camera_radius),
      ),
      opacity: (opacity(shown.0), opacity(shown.1)),
      camera_front: clip.camera_front(),
    })
  }
}

/// The placement the scenes give the frame `source_ms` into the recording,
/// or `None` outside every clip, in a stretch the timeline cut away, and in a
/// scene that stands idle.
fn placement_at(
  clips: &[RecordingSceneClip],
  ranges: &[TimelineRange],
  source_ms: u64,
  base: &Base,
) -> Option<Placement> {
  let index = clips
    .iter()
    .position(|clip| covers(ranges, [clip.start_ms, clip.end_ms], source_ms))?;
  let clip = &clips[index];
  let target = base.target(clip)?;
  let (elapsed, length) = output_progress(ranges, [clip.start_ms, clip.end_ms], source_ms)?;
  // A scene at either end of the video holds there: there is nothing before
  // it to arrive from, or after it to leave for. Its other transition may then
  // take the whole clip.
  let (starts_video, ends_video) = reaches_video_ends(ranges, [clip.start_ms, clip.end_ms]);
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
/// camera's source size, as `clips` have them `source_ms` into the recording,
/// timed on `ranges`. Both are left as they are outside every clip, which the
/// answer says: `true` where a scene placed them. `frame_ms` is how much
/// source time the frame covers: where the panes moved over it, `output`
/// carries that motion for the compositor to blur; a still passes zero.
pub(crate) fn arrange(
  clips: &[RecordingSceneClip],
  ranges: &[TimelineRange],
  source_ms: u64,
  frame_ms: f32,
  output: &mut ScreenshotOutputSettings,
  camera: Option<(&mut CameraOverlaySettings, (u32, u32))>,
) -> bool {
  if clips.is_empty() || !output.has_placement() {
    return false;
  }
  let camera = camera.filter(|(_, (width, height))| *width > 0 && *height > 0);
  let base = Base::of(
    output,
    camera
      .as_ref()
      .map(|(overlay, (width, height))| (&**overlay, f64::from(*width) / f64::from(*height))),
  );
  let Some(placed) = placement_at(clips, ranges, source_ms, &base) else {
    return false;
  };
  if frame_ms > 0.0 {
    // Before its first clip the frame was in the composition outside them.
    let opened_ms = source_ms.saturating_sub(frame_ms.round() as u64);
    let opened = placement_at(clips, ranges, opened_ms, &base).unwrap_or(base.placement);
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

/// What a drag on the camera reframes `source_ms` into the recording: the
/// index of the clip it falls in, the box that clip settles the camera in,
/// and the framing it shows there. `None` outside every clip, in a scene
/// that stands idle, and in one that hides the camera.
pub(crate) fn camera_target(
  clips: &[RecordingSceneClip],
  source_ms: u64,
  output: &ScreenshotOutputSettings,
  overlay: &CameraOverlaySettings,
  camera: (u32, u32),
) -> Option<(usize, Rect, SceneFraming)> {
  if camera.0 == 0 || camera.1 == 0 || !output.has_placement() {
    return None;
  }
  let aspect = f64::from(camera.0) / f64::from(camera.1);
  let base = Base::of(output, Some((overlay, aspect)));
  let index = clips
    .iter()
    .position(|clip| clip.start_ms <= source_ms && source_ms < clip.end_ms)?;
  let clip = &clips[index];
  let target = base.target(clip)?;
  if target.opacity.1 == 0.0 {
    return None;
  }
  let frame = target.frame;
  let framing = clip.camera.unwrap_or_else(|| {
    if clip.needs_camera() {
      SceneFraming::WHOLE
    } else {
      SceneFraming::of_camera(overlay, aspect)
    }
  });
  Some((index, frame, framing.within(frame, aspect)))
}
