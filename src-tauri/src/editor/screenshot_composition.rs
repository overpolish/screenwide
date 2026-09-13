// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The canvas's own background picture, filled to it. `None` whenever the
/// background is a painted one, or the file behind it can no longer be read,
/// in which case the compositor's solid colour stands in.
#[cfg(target_os = "macos")]
pub(super) fn background_image_layer(canvas: &ScreenshotOutputSettings) -> Option<CapturedImage> {
  if canvas.background_type != "image" {
    return None;
  }
  let picture = crate::screenshots::background_image_canvas(
    canvas.background_image_path.as_deref()?,
    canvas.width,
    canvas.height,
  )?;
  let (width, height) = picture.dimensions();
  Some(CapturedImage {
    height,
    rgba: picture.into_raw(),
    width,
  })
}

pub(super) fn compose_screenshot_workspace(
  app: &AppHandle,
  items: &[ScreenshotItem],
  output: &ScreenshotWorkspaceOutputSettings,
) -> Result<CapturedImage, String> {
  #[cfg(not(target_os = "windows"))]
  let _ = app;
  let ordered_items = output
    .items
    .iter()
    .filter_map(|item_output| items.iter().find(|item| item.id == item_output.id))
    .collect::<Vec<_>>();
  #[cfg(not(target_os = "windows"))]
  let first = ordered_items
    .first()
    .copied()
    .ok_or_else(|| "The screenshot workspace is empty".to_owned())?;
  #[cfg(target_os = "macos")]
  {
    // A chosen picture is the one background the compositor cannot paint: it
    // draws from uniforms, not from a texture. So the layers are composed
    // over nothing, the picture is filled to the canvas here, and the two are
    // put together. The shadow survives that, since a background-less layer
    // carries it as its own alpha, and the canvas corners are rounded last,
    // exactly where the shader would have rounded them.
    let picture = background_image_layer(&output.canvas);
    let mut composed = crate::screenshots::compose_output_layers(
      &first.image,
      &output.output_for(first),
      0.0,
      true,
      None,
      None,
      None,
      None,
      false,
      picture.is_some(),
    )?;
    for item in &ordered_items[1..] {
      let layer = crate::screenshots::compose_output_layers(
        &item.image,
        &output.output_for(item),
        0.0,
        true,
        None,
        None,
        None,
        None,
        false,
        true,
      )?;
      composed = crate::screenshots::alpha_composite(&composed, &layer)?;
    }
    let Some(picture) = picture else {
      return Ok(composed);
    };
    Ok(crate::screenshots::rounded_corners(
      &crate::screenshots::alpha_composite(&picture, &composed)?,
      output.canvas.background_radius_percent,
    ))
  }
  #[cfg(target_os = "windows")]
  {
    let window = app
      .get_webview_window(EditorKind::Screenshot.window_label().as_str())
      .ok_or_else(|| "The editor window is unavailable".to_owned())?;
    let surface = preview_platform::RecordingPreviewSurface::from_window(&window)?;
    let layers = ordered_items
      .iter()
      .map(|item| (&item.image, output.output_for(item)))
      .collect::<Vec<_>>();
    surface.compose_screenshot_layers_to_image(&layers)
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    compose_screenshot(&first.image, &output.output_for(first))
  }
}
