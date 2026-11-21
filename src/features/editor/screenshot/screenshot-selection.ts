// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CaptureSource, cornerRadiusFramePercent } from "../layer-measure";
import { UPRIGHT_ROTATION, withShareOfTurn } from "../layer-rotation";
import { openingCropChrome } from "../preview/opening-crop-chrome";

import {
  ScreenshotLayout,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  screenshotLayout,
} from "./screenshot-output";

import type { CropTurn } from "../preview/use-crop-turn";

/** Convert output-pixel layout into the pane fractions native OSCs consume. */
export function normalizedScreenshotSelection(
  layout: ScreenshotLayout,
  output: { height: number; width: number },
  mode: "crop" | "select" = "select",
) {
  const height = Math.max(1, output.height);
  const width = Math.max(1, output.width);
  const fractions = (box: ScreenshotLayout["crop"]) => ({
    height: box.height / height,
    width: box.width / width,
    x: box.x / width,
    y: box.y / height,
  });
  const image = fractions(layout.image);
  const bounds = fractions(layout.crop);
  const sourceCrop = fractions(layout.sourceCrop);
  return {
    image,
    // The crop overlay is bounded by the whole uncropped picture it is cut
    // from; the select overlay is bounded by nothing.
    recenterBounds: mode === "crop" ? image : undefined,
    rect: mode === "select" ? bounds : sourceCrop,
  };
}

/** How the select tool's gizmo finds a layer turned: by its own turn, upright
 * where it has none. The crop tool reframes the layer upright, with no gizmo,
 * so its selection carries none. */
const screenshotLayerTurn = (
  output: ScreenshotOutputSettings,
  cropMode: boolean,
) => (cropMode ? {} : { rotation: output.rotation ?? UPRIGHT_ROTATION });

/** The selection the native overlay draws round the chosen layer: its frame,
 * or its crop while the crop tool shows it, turned as far as the crop's ease
 * (`cropTurn`) has brought it, the crop's chrome opening with it
 * (`openingCropChrome`), which keeps the turn on the crop only while the
 * gizmo fades. */
export const screenshotItemSelection = ({
  cropKeepsShape,
  cropMode,
  cropTurn,
  item,
  itemOutput,
  paneIndex,
  target,
}: {
  cropKeepsShape: boolean;
  cropMode: boolean;
  cropTurn: CropTurn;
  item: CaptureSource & { isBlank?: boolean };
  itemOutput: ScreenshotOutputSettings;
  paneIndex: number;
  target: { height: number; width: number };
}) =>
  openingCropChrome(
    {
      cropKeepsShape: cropMode && cropKeepsShape,
      cropMode,
      // A blank canvas's own layer draws no frame and is never pressed as a
      // picture (`screenshot-blank.ts`).
      isBlank: item.isBlank === true,
      paneIndex,
      radiusPercent: cornerRadiusFramePercent(itemOutput, item),
      rotation:
        withShareOfTurn(itemOutput, cropTurn.share).rotation ??
        UPRIGHT_ROTATION,
      ...normalizedScreenshotSelection(
        screenshotLayout(item, itemOutput),
        target,
        cropMode ? "crop" : "select",
      ),
    },
    cropTurn,
  );

/** One selectable target per workspace pane, in pane order, for the native
 * select and crop overlays to hit-test. Panes whose item is gone are skipped. */
export function screenshotSelectionTargets(
  items: (CaptureSource & { id: number; isBlank?: boolean })[],
  workspaceOutput: ScreenshotWorkspaceOutputSettings,
  target: { cropMode: boolean; height: number; width: number },
) {
  const { cropMode } = target;
  return workspaceOutput.items.flatMap((itemOutput, paneIndex) => {
    const item = items.find((candidate) => candidate.id === itemOutput.id);
    if (!item) return [];
    const layout = screenshotLayout(item, itemOutput.output);
    return [
      {
        cropMode,
        isBlank: item.isBlank === true,
        paneIndex,
        radiusPercent: cornerRadiusFramePercent(itemOutput.output, item),
        ...screenshotLayerTurn(itemOutput.output, cropMode),
        ...normalizedScreenshotSelection(
          layout,
          target,
          cropMode ? "crop" : "select",
        ),
      },
    ];
  });
}
