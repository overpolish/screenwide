// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Ruler at work on a picture rather than the frozen desktop, for a
//! window that shows what the Ruler does. It is a document of its own,
//! apart from the session the shortcut starts, fed the same pointer input.

use screenwide_imaging::CapturedImage;
use screenwide_osc::desktop::DesktopDisplay;
use screenwide_osc::geometry::{Point, Size};
use screenwide_osc::protocol::{InputModifiers, InputPhase, OscResult};

use super::{dispatch_input, visual_result, RulerState};
use crate::app_windows::screenshot_region::region_osc::preview::PREVIEW_DISPLAY_ID as DISPLAY_ID;

pub(crate) struct RulerPreview {
  state: RulerState,
  generation: u64,
  display: DesktopDisplay,
  picture: CapturedImage,
}

impl RulerPreview {
  /// Measures `picture`, shown at `size` points. Element detection runs
  /// here, so this belongs on a thread that may wait for it.
  pub(crate) fn new(picture: CapturedImage, size: Size) -> Self {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: DISPLAY_ID,
      origin: Point::default(),
      size,
      scale: f64::from(picture.width) / size.width.max(1.0),
    };
    let preview = Self {
      state,
      generation,
      display,
      picture,
    };
    preview.reset();
    preview
  }

  /// Clears every measurement, as a new session would.
  pub(crate) fn reset(&self) {
    self.state.install(
      self.generation,
      &[self.display],
      &[(DISPLAY_ID, self.picture.clone())],
    );
    self.state.analyze_boxes(self.generation);
  }

  pub(crate) fn state(&self) -> &RulerState {
    &self.state
  }

  /// Applies `phase` at `point`, in the picture's points, and answers the
  /// result the overlay would draw.
  pub(crate) fn input(&self, phase: InputPhase, point: Point) -> Option<OscResult> {
    let dispatch = dispatch_input(&self.state, phase, point, InputModifiers::default())?;
    Some(visual_result(
      &self.state,
      dispatch.visual,
      dispatch.cursor,
      dispatch.handle,
    ))
  }
}
