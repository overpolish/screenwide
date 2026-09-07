// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { resetCommittedScreenshotCrop } from "../screenshot-crop";
import {
  resetScreenshotTransform,
  resizeScreenshotWorkspaceCentered,
  ScreenshotOutputSettings,
} from "../screenshot-output";

import { ScreenshotSectionProps } from "./editor-preview-section-props";
import { PreviewToolReset } from "./preview-tool-reset";
import { RecordingCanvasTool } from "./recording-crop-toggle";

export function ScreenshotToolReset({
  artifact,
  isSaving,
  onCanvasResize,
  onOutputChange,
  onRecenterReset,
  screenshotOutput,
  selectedItem,
  selectedOutput,
  tool,
}: Pick<
  ScreenshotSectionProps,
  | "artifact"
  | "isSaving"
  | "onCanvasResize"
  | "onOutputChange"
  | "screenshotOutput"
> & {
  onRecenterReset: () => void;
  selectedOutput: ScreenshotOutputSettings | null;
  tool: RecordingCanvasTool;
  selectedItem?: ScreenshotSectionProps["artifact"]["items"][number];
}) {
  const canReset =
    tool === "canvas"
      ? Boolean(screenshotOutput && onCanvasResize)
      : Boolean(selectedItem && selectedOutput && onOutputChange);
  const reset = () => {
    if (isSaving || !canReset) return;
    if (tool === "canvas") {
      if (!screenshotOutput) return;
      onCanvasResize?.(
        resizeScreenshotWorkspaceCentered({
          height: artifact.height,
          settings: screenshotOutput,
          sources: artifact.items,
          width: artifact.width,
        }),
      );
    } else if (tool === "recenter") {
      onRecenterReset();
    } else if (selectedItem && selectedOutput && tool) {
      onOutputChange?.(
        tool === "select"
          ? resetScreenshotTransform(selectedOutput, selectedItem)
          : resetCommittedScreenshotCrop(selectedOutput, selectedItem),
        selectedItem.id,
      );
    }
  };
  return (
    <PreviewToolReset
      isDisabled={isSaving || !canReset}
      onReset={reset}
      tool={tool}
    />
  );
}
