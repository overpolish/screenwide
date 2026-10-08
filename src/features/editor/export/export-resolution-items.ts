// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { appLocale, t } from "../../../i18n/i18n";
import {
  cameraResolutionScales,
  resolutionScales,
  scaledDimensions,
  scaledVideoDimensions,
} from "../resolution";
import { EditorArtifact } from "../types";

import type { ExportResolutionItem } from "./export-resolution-select";

type RecordingArtifact = Extract<EditorArtifact, { kind: "recording" }>;

/** Labels are ratios of the first (source) scale, matching the old inspector. */
const scaleItem = (
  { height, width }: { height: number; width: number },
  scale: number,
  sourceScale: number,
): ExportResolutionItem => ({
  height,
  id: scale.toString(),
  label:
    scale === sourceScale
      ? t("editor-export-scale-original")
      : t("editor-export-scale", {
          ratio: new Intl.NumberFormat(appLocale().formatLocale, {
            maximumFractionDigits: 2,
          }).format(scale / sourceScale),
        }),
  width,
});

/** The first scale is the captured one, so it is the "Original" choice.
 * The sizes are those of the canvas the editor's Dimensions setting gives the
 * video, when it has one; the source size only stands in before then. */
export const outputResolutionItems = (
  artifact: RecordingArtifact,
  output?: { height: number; width: number } | null,
) =>
  resolutionScales(artifact).map((scale, _index, scales) =>
    scaleItem(
      output
        ? scaledVideoDimensions({
            height: output.height,
            scale,
            sourceScale: scales[0],
            width: output.width,
          })
        : scaledDimensions(artifact, scale),
      scale,
      scales[0],
    ),
  );

/** As above: the camera's canvas from the Dimensions setting when set. */
export const cameraResolutionItems = (
  camera: { height: number; width: number },
  output?: { height: number; width: number } | null,
) =>
  cameraResolutionScales.map((scale) =>
    scaleItem(
      scaledVideoDimensions({
        height: (output ?? camera).height,
        scale,
        sourceScale: 100,
        width: (output ?? camera).width,
      }),
      scale,
      100,
    ),
  );
