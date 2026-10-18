// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn compose_screenshot_workspace(
  items: &[ScreenshotItem],
  output: &ScreenshotWorkspaceOutputSettings,
) -> Result<CapturedImage, String> {
  let layers = output
    .items
    .iter()
    .filter_map(|item_output| items.iter().find(|item| item.id == item_output.id))
    .map(|item| (&item.image, output.output_for(item, items)))
    .collect::<Vec<_>>();
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
    screenwide_compositor::compositor::still::compose_layers(&layers)
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let (image, settings) = layers
      .first()
      .ok_or_else(|| "The screenshot workspace is empty".to_owned())?;
    compose_screenshot(image, settings)
  }
}
