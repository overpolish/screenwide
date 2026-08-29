// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use image::{imageops::FilterType, RgbaImage};

use crate::desktop_capture::{CapturePiece, CapturePlan};

use super::CapturedImage;

pub fn compose(
  plan: &CapturePlan,
  captured: Vec<(CapturePiece, CapturedImage)>,
) -> Result<CapturedImage, String> {
  let mut output = RgbaImage::new(plan.width, plan.height);
  for (piece, image) in captured {
    let source = RgbaImage::from_raw(image.width, image.height, image.rgba)
      .ok_or_else(|| "A display capture returned invalid pixels".to_owned())?;
    let resized = if source.dimensions() == (piece.destination.width, piece.destination.height) {
      source
    } else {
      image::imageops::resize(
        &source,
        piece.destination.width,
        piece.destination.height,
        FilterType::Lanczos3,
      )
    };
    image::imageops::overlay(
      &mut output,
      &resized,
      i64::from(piece.destination.x),
      i64::from(piece.destination.y),
    );
  }
  Ok(CapturedImage {
    width: output.width(),
    height: output.height(),
    rgba: output.into_raw(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn leaves_desktop_gaps_transparent() {
    let displays = [
      crate::desktop_capture::DesktopDisplay {
        id: 1,
        x: 0.0,
        y: 0.0,
        width: 1800.0,
        height: 1169.0,
        scale: 2.0,
      },
      crate::desktop_capture::DesktopDisplay {
        id: 2,
        x: 1800.0,
        y: 89.0,
        width: 1920.0,
        height: 1080.0,
        scale: 1.0,
      },
    ];
    let plan = crate::desktop_capture::plan(
      &displays,
      1,
      crate::recording::Region {
        position: tauri::LogicalPosition::new(1700.0, 0.0),
        size: tauri::LogicalSize::new(300.0, 100.0),
      },
      crate::desktop_capture::OutputLimits::UNBOUNDED,
    )
    .unwrap();
    let piece = plan.pieces[0];
    let red = CapturedImage {
      rgba: [255, 0, 0, 255].repeat(200 * 200),
      width: 200,
      height: 200,
    };
    let output = compose(&plan, vec![(piece, red)]).unwrap();
    assert_eq!(&output.rgba[0..4], &[255, 0, 0, 255]);
    let gap = ((50 * output.width + 250) * 4) as usize;
    assert_eq!(&output.rgba[gap..gap + 4], &[0, 0, 0, 0]);
  }
}
