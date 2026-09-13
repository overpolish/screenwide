// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[path = "snapshot/tests.rs"]
mod tests;

use std::{collections::HashMap, sync::Mutex};

use crate::{
  desktop_capture::{self, OutputLimits, PixelRect},
  osc::desktop::DesktopDisplay as OscDesktopDisplay,
  osc::geometry::Rect,
  recording::Region,
  screenshots::{self, CapturedImage},
};

struct MonitorSnapshot {
  image: CapturedImage,
}

#[derive(Default)]
pub(super) struct Session {
  pub(super) generation: u64,
  monitors: HashMap<u32, MonitorSnapshot>,
  selected: Option<CapturedImage>,
  pub(super) selection: Option<Rect>,
  pub(super) text: Option<super::text_selection::TextSelection>,
  pub(super) pressed_qr: Option<usize>,
}

#[derive(Default)]
pub struct TextRecognitionState(pub(super) Mutex<Session>);

impl TextRecognitionState {
  pub(super) fn is_active(&self) -> bool {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    !session.monitors.is_empty() || session.selected.is_some()
  }

  pub(super) fn active_generation(&self) -> Option<u64> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    (!session.monitors.is_empty() || session.selected.is_some()).then_some(session.generation)
  }

  pub(super) fn begin(&self) -> u64 {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.generation = session.generation.wrapping_add(1);
    session.monitors.clear();
    session.selected = None;
    session.selection = None;
    session.text = None;
    session.pressed_qr = None;
    session.generation
  }

  pub(super) fn cancel(&self) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.generation = session.generation.wrapping_add(1);
    let had_capture = !session.monitors.is_empty() || session.selected.is_some();
    session.monitors.clear();
    session.selected = None;
    session.selection = None;
    session.text = None;
    session.pressed_qr = None;
    had_capture
  }

  pub(super) fn install(
    &self,
    generation: u64,
    monitors: impl IntoIterator<Item = (u32, f64, CapturedImage)>,
  ) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.generation != generation {
      return false;
    }
    session.monitors = monitors
      .into_iter()
      .map(|(id, _scale, image)| (id, MonitorSnapshot { image }))
      .collect();
    true
  }

  pub(super) fn recognition_input(&self) -> Option<(u64, CapturedImage)> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session
      .selected
      .clone()
      .map(|image| (session.generation, image))
  }

  pub(super) fn select_desktop_region(
    &self,
    displays: &[OscDesktopDisplay],
    anchor_id: u32,
    region: Rect,
  ) -> Result<CapturedImage, String> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let displays = displays
      .iter()
      .map(|display| desktop_capture::DesktopDisplay {
        id: display.id,
        x: display.origin.x,
        y: display.origin.y,
        width: display.size.width,
        height: display.size.height,
        scale: display.scale,
      })
      .collect::<Vec<_>>();
    let plan = desktop_capture::plan(
      &displays,
      anchor_id,
      Region {
        position: tauri::LogicalPosition::new(region.origin.x, region.origin.y),
        size: tauri::LogicalSize::new(region.size.width, region.size.height),
      },
      OutputLimits::UNBOUNDED,
    )?;
    let pieces = plan
      .pieces
      .iter()
      .copied()
      .map(|piece| {
        let snapshot = session
          .monitors
          .get(&piece.display_id)
          .ok_or_else(|| "A frozen monitor image is no longer available".to_owned())?;
        crop_pixels(&snapshot.image, piece.source_pixels).map(|image| (piece, image))
      })
      .collect::<Result<Vec<_>, _>>()?;
    let selected = screenshots::desktop::compose(&plan, pieces)?;
    session.selection = Some(Rect::from_xywh(
      plan.desktop_region.x,
      plan.desktop_region.y,
      plan.desktop_region.width,
      plan.desktop_region.height,
    ));
    session.selected = Some(selected.clone());
    session.text = None;
    session.pressed_qr = None;
    session.generation = session.generation.wrapping_add(1);
    Ok(selected)
  }

  pub(super) fn visual_snapshot(&self, generation: u64) -> Option<super::visual::VisualSnapshot> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.generation != generation {
      return None;
    }
    session
      .selection
      .zip(session.text.as_ref())
      .map(|(selection, model)| {
        super::visual::snapshot(selection, model.result(), &model.rectangles())
      })
  }

  pub(super) fn is_current_generation(&self, generation: u64) -> bool {
    self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .generation
      == generation
  }
}

fn crop_pixels(image: &CapturedImage, rect: PixelRect) -> Result<CapturedImage, String> {
  if rect.x + rect.width > image.width || rect.y + rect.height > image.height {
    return Err("The frozen monitor crop is outside its captured image".to_owned());
  }
  let source_stride = image.width as usize * 4;
  let target_stride = rect.width as usize * 4;
  let mut rgba = vec![0_u8; target_stride * rect.height as usize];
  for row in 0..rect.height as usize {
    let source_start = (rect.y as usize + row) * source_stride + rect.x as usize * 4;
    let target_start = row * target_stride;
    rgba[target_start..target_start + target_stride]
      .copy_from_slice(&image.rgba[source_start..source_start + target_stride]);
  }
  Ok(CapturedImage {
    rgba,
    width: rect.width,
    height: rect.height,
  })
}
