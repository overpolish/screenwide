// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNativeCropRequest } from "../preview/use-native-crop-request";
import { usePublishCropTool } from "../tools/crop/crop-panel-handlers";

import type { screenshotChoices } from "./screenshot-choices";

/** The screenshot's crop asked for on the picture, a layer double-clicked
 * (its pane, in workspace order) or a press past the picture being cropped,
 * and from the islands, the layer's crop button and the crop's Done. */
export function useScreenshotCropAsks(
  choices: ReturnType<typeof screenshotChoices>,
  items: readonly { id: number }[],
) {
  useNativeCropRequest((paneIndex) => {
    choices.cropLayer(
      paneIndex === null ? null : (items[paneIndex]?.id ?? null),
    );
  });
  usePublishCropTool((isCropping) => {
    if (isCropping) choices.takeUp("crop");
    else choices.cropLayer(null);
  });
}
