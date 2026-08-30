// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn crop_pixels_uses_the_planned_device_pixel_rect() {
    let image = CapturedImage {
      height: 4,
      rgba: (0_u8..64).collect(),
      width: 4,
    };
    let cropped = crop_pixels(
      &image,
      PixelRect {
        x: 1,
        y: 1,
        width: 2,
        height: 2,
      },
    )
    .unwrap();

    assert_eq!((cropped.width, cropped.height), (2, 2));
    assert_eq!(
      cropped.rgba,
      [&image.rgba[20..28], &image.rgba[36..44]].concat()
    );
  }

  #[test]
  fn desktop_selection_composes_across_the_monitor_boundary() {
    let state = TextRecognitionState::default();
    let generation = state.begin();
    assert!(state.install(
      generation,
      [
        (
          1,
          1.0,
          CapturedImage {
            rgba: [[255, 0, 0, 255], [0, 255, 0, 255]].concat(),
            width: 2,
            height: 1,
          },
        ),
        (
          2,
          1.0,
          CapturedImage {
            rgba: [[0, 0, 255, 255], [255, 255, 0, 255]].concat(),
            width: 2,
            height: 1,
          },
        ),
      ]
    ));
    let displays = [
      OscDesktopDisplay {
        id: 1,
        origin: crate::osc::geometry::Point { x: 0.0, y: 0.0 },
        size: crate::osc::geometry::Size {
          width: 2.0,
          height: 1.0,
        },
        scale: 1.0,
      },
      OscDesktopDisplay {
        id: 2,
        origin: crate::osc::geometry::Point { x: 2.0, y: 0.0 },
        size: crate::osc::geometry::Size {
          width: 2.0,
          height: 1.0,
        },
        scale: 1.0,
      },
    ];
    let selected = state
      .select_desktop_region(&displays, 1, Rect::from_xywh(1.0, 0.0, 2.0, 1.0))
      .unwrap();

    assert_eq!((selected.width, selected.height), (2, 1));
    assert_eq!(selected.rgba, [[0, 255, 0, 255], [0, 0, 255, 255]].concat());
    assert_eq!(state.recognition_input().unwrap().0, generation + 1);
  }

  #[test]
  fn a_new_session_rejects_an_older_recognition_result() {
    let state = TextRecognitionState::default();
    let old_generation = state.begin();
    let current_generation = state.begin();

    assert!(!state.is_current_generation(old_generation));
    assert!(state.is_current_generation(current_generation));
  }

  #[test]
  fn dismissed_sessions_cannot_restart_from_topology_callbacks() {
    let state = TextRecognitionState::default();
    let generation = state.begin();
    assert!(state.install(
      generation,
      [(
        1,
        1.0,
        CapturedImage {
          rgba: vec![0, 0, 0, 255],
          width: 1,
          height: 1,
        }
      )],
    ));
    assert_eq!(state.active_generation(), Some(generation));

    state.cancel();

    assert_eq!(state.active_generation(), None);
    assert!(!state.is_current_generation(generation));
  }
}
