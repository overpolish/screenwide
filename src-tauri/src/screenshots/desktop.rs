// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use image::{imageops::FilterType, RgbaImage};

use screenwide_capture::desktop_capture::{CapturePiece, CapturePlan};

use super::CapturedImage;

/// Joins the pieces of an area captured across displays into one picture,
/// each where it lands, leaving the gaps between displays transparent.
/// Displays never overlap, so each piece is copied into place, row by row,
/// rather than blended over what is there.
pub fn compose(
  plan: &CapturePlan,
  mut captured: Vec<(CapturePiece, CapturedImage)>,
) -> Result<CapturedImage, String> {
  // An area on one display, at that display's scale, is the piece itself.
  if let [(piece, image)] = captured.as_slice() {
    let destination = piece.destination;
    if (destination.x, destination.y) == (0, 0)
      && (destination.width, destination.height) == (plan.width, plan.height)
      && (image.width, image.height) == (plan.width, plan.height)
    {
      return Ok(captured.swap_remove(0).1);
    }
  }
  let output_stride = plan.width as usize * 4;
  let mut output = vec![0_u8; output_stride * plan.height as usize];
  for (piece, image) in captured {
    let destination = piece.destination;
    let source = if (image.width, image.height) == (destination.width, destination.height) {
      image.rgba
    } else {
      let source = RgbaImage::from_raw(image.width, image.height, image.rgba)
        .ok_or_else(|| "A display capture returned invalid pixels".to_owned())?;
      image::imageops::resize(
        &source,
        destination.width,
        destination.height,
        FilterType::Lanczos3,
      )
      .into_raw()
    };
    let source_stride = destination.width as usize * 4;
    if source.len() < source_stride * destination.height as usize {
      return Err("A display capture returned invalid pixels".to_owned());
    }
    let left = (destination.x.min(plan.width) as usize) * 4;
    let width = output_stride.saturating_sub(left).min(source_stride);
    let rows = (destination
      .height
      .min(plan.height.saturating_sub(destination.y))) as usize;
    for row in 0..rows {
      let target = (destination.y as usize + row) * output_stride + left;
      let from = row * source_stride;
      output[target..target + width].copy_from_slice(&source[from..from + width]);
    }
  }
  Ok(CapturedImage {
    width: plan.width,
    height: plan.height,
    rgba: output,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn leaves_desktop_gaps_transparent() {
    let displays = [
      screenwide_capture::desktop_capture::DesktopDisplay {
        id: 1,
        x: 0.0,
        y: 0.0,
        width: 1800.0,
        height: 1169.0,
        scale: 2.0,
      },
      screenwide_capture::desktop_capture::DesktopDisplay {
        id: 2,
        x: 1800.0,
        y: 89.0,
        width: 1920.0,
        height: 1080.0,
        scale: 1.0,
      },
    ];
    let plan = screenwide_capture::desktop_capture::plan(
      &displays,
      1,
      screenwide_capture::Region {
        position: tauri::LogicalPosition::new(1700.0, 0.0),
        size: tauri::LogicalSize::new(300.0, 100.0),
      },
      screenwide_capture::desktop_capture::OutputLimits::UNBOUNDED,
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
