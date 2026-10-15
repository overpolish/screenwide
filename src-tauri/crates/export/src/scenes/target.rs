// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a clip settles the panes, measured from the composition outside
//! every scene, which a scene arrives from and leaves for.

use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::framing::{drawn_frame, SceneFraming};
use screenwide_scenes::geometry::{preset_panes, Rect};
use screenwide_scenes::model::{RecordingSceneClip, RecordingScenePreset};
use screenwide_scenes::placement::Placement;
use screenwide_scenes::CameraOverlaySettings;

/// What a frame is arranged from: the composition outside every scene, the
/// canvas, and the camera picture's aspect where a camera is drawn into it.
pub(super) struct Base {
  pub(super) placement: Placement,
  pub(super) canvas: (f64, f64),
  pub(super) camera_aspect: Option<f64>,
}

impl Base {
  pub(super) fn of(
    output: &ScreenshotOutputSettings,
    camera: Option<(&CameraOverlaySettings, f64)>,
  ) -> Self {
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
        rotation: output.rotation.unwrap_or_default(),
      },
      canvas: (f64::from(output.width), f64::from(output.height)),
      camera_aspect: camera.map(|(_, aspect)| aspect),
    }
  }

  /// Where `clip` settles the panes, or `None` for an arrangement that needs
  /// a camera where none is drawn into the picture: that scene stands idle.
  /// A pane the preset hides keeps the recording's own place, unseen.
  pub(super) fn target(&self, clip: &RecordingSceneClip) -> Option<Placement> {
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
      rotation: clip.rotation.unwrap_or_default(),
    })
  }
}
