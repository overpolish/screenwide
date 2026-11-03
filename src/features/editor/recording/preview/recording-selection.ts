// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ScreenshotOutputSettings,
  screenshotLayout,
  screenshotOutputDimensions,
} from "../../screenshot/screenshot-output";
import { normalizedScreenshotSelection } from "../../screenshot/screenshot-selection";
import {
  keepsShape,
  recordingCropAspectKey,
} from "../../tools/crop/crop-aspect-store";

import { RecordingCanvasTool } from "./recording-canvas-tool";

import type { AnnotationKind } from "../../../annotations/types";
import type { CropAspect } from "../../island/crop-aspect-choice";
import type { RecordingPreviewSelection } from "../use-recording-preview-surface";

type SelectionTool = Exclude<
  RecordingCanvasTool,
  AnnotationKind | "canvas" | "marquee" | "scene"
>;

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
    screenshotLayout(source, output),
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
    rect: selection.rect,
  };
}

/** `selection`, where it is a crop window, told whether its handles keep
 * its shape: as a screenshot's crop does, wherever a shape is chosen for it
 * (`aspects`, by `recordingCropAspectKey`), the screen's over the scene
 * `sceneId` by that scene's own. */
export const withCropShape = (
  selection: RecordingPreviewSelection,
  aspects: Partial<Record<string, CropAspect>>,
  sceneId: string | null,
): RecordingPreviewSelection =>
  selection.cropMode
    ? {
        ...selection,
        cropKeepsShape: keepsShape(
          aspects[
            recordingCropAspectKey(
              (selection.layerId ?? selection.paneIndex) === 1
                ? "camera"
                : "primary",
              sceneId,
            )
          ],
        ),
      }
    : selection;
