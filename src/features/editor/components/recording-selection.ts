// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ScreenshotOutputSettings,
  screenshotLayout,
  screenshotOutputDimensions,
} from "../screenshot-output";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import { normalizedScreenshotSelection } from "./screenshot-selection";

type SelectionTool = Exclude<RecordingCanvasTool, "canvas" | null>;

/** Build the native OSC payload for an ordinary screen/camera output pane. */
export function normalizedRecordingSelection({
  mode,
  output,
  paneIndex,
  source,
}: {
  mode: SelectionTool;
  output: ScreenshotOutputSettings;
  paneIndex: number;
  source: { height: number; width: number };
}) {
  const dimensions = screenshotOutputDimensions(output);
  const selection = normalizedScreenshotSelection(
    screenshotLayout(source, dimensions, output),
    dimensions,
    mode,
  );
  return {
    cropMode: mode === "crop",
    image: selection.image,
    layerId: paneIndex,
    paneIndex,
    radiusPercent: output.radiusPercent,
    recenterBounds: selection.recenterBounds,
    recenterMode: mode === "recenter",
    rect: selection.rect,
  };
}
