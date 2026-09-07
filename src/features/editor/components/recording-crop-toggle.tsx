// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleDotDashed, Crop, MousePointer2, ScanSquare } from "lucide-react";

import {
  cameraOverlayForDimensions,
  defaultCameraOverlay,
} from "../recording-export-settings";
import {
  RecordingOutputSettings,
  resetScreenshotCrop,
  resetScreenshotLayout,
  resetScreenshotTransform,
} from "../screenshot-output";
import {
  CameraOverlaySettings,
  RecordingPreviewPane,
  RecordingVideoTrackId,
} from "../types";

import { PreviewToolReset } from "./preview-tool-reset";
import { PreviewToolToggle } from "./preview-tool-toggle";

export type RecordingCanvasTool =
  "canvas" | "crop" | "recenter" | "select" | null;

export function RecordingCanvasTools({
  activeTrack,
  bakeCamera,
  cameraPane,
  isEnabled,
  isFrameEnabled = isEnabled,
  isRecenterEnabled = false,
  isSelectEnabled = isEnabled,
  onCameraOverlayReset,
  onChange,
  onRecenterReset,
  onToolChange,
  outputs,
  screenPane,
  tool,
}: {
  activeTrack: RecordingVideoTrackId | null;
  bakeCamera: boolean;
  isEnabled: boolean;
  onToolChange: (tool: RecordingCanvasTool) => void;
  tool: RecordingCanvasTool;
  cameraPane?: RecordingPreviewPane;
  isFrameEnabled?: boolean;
  isRecenterEnabled?: boolean;
  isSelectEnabled?: boolean;
  onCameraOverlayReset?: (settings: CameraOverlaySettings) => void;
  onChange?: (
    track: RecordingVideoTrackId,
    settings: RecordingOutputSettings[RecordingVideoTrackId],
  ) => void;
  onRecenterReset?: () => void;
  outputs?: RecordingOutputSettings;
  screenPane?: RecordingPreviewPane;
}) {
  const resetEnabled =
    tool === "canvas"
      ? isFrameEnabled
      : tool === "select"
        ? isSelectEnabled
        : tool === "recenter"
          ? isRecenterEnabled
          : tool === "crop" && isEnabled;
  const targetTrack = bakeCamera && tool === "canvas" ? "primary" : activeTrack;
  const canReset =
    tool === "recenter"
      ? Boolean(onRecenterReset)
      : activeTrack === "camera" && bakeCamera && tool !== "canvas"
        ? Boolean(onCameraOverlayReset)
        : Boolean(
            onChange &&
            outputs &&
            targetTrack &&
            (targetTrack === "primary" ? screenPane : cameraPane),
          );
  const reset = () => {
    if (!tool || !resetEnabled || !canReset) return;
    if (tool === "recenter") {
      onRecenterReset?.();
      return;
    }
    if (activeTrack === "camera" && bakeCamera && tool !== "canvas") {
      // The reset must be computed from the real output and camera geometry:
      // generic 16:9 defaults land the crop frame outside the camera image
      // for other aspect ratios, and the compositor's clamped rendering then
      // no longer matches the on-screen controls.
      onCameraOverlayReset?.(
        outputs && cameraPane
          ? cameraOverlayForDimensions({
              cameraHeight: cameraPane.sourceHeight,
              cameraWidth: cameraPane.sourceWidth,
              screenHeight: outputs.primary.height,
              screenWidth: outputs.primary.width,
            })
          : defaultCameraOverlay(),
      );
      return;
    }
    if (!activeTrack || !outputs) return;
    const targetTrack =
      bakeCamera && tool === "canvas" ? "primary" : activeTrack;
    const pane = targetTrack === "primary" ? screenPane : cameraPane;
    if (!pane) return;
    const source = { height: pane.sourceHeight, width: pane.sourceWidth };
    const current = outputs[targetTrack];
    const next =
      tool === "canvas"
        ? resetScreenshotLayout(
            { ...current, height: source.height, width: source.width },
            source,
          )
        : tool === "select"
          ? resetScreenshotTransform(current, source)
          : resetScreenshotCrop(current, source);
    onChange?.(targetTrack, next);
  };
  return (
    <>
      <PreviewToolToggle
        isDisabled={!isSelectEnabled}
        isSelected={tool === "select" && isSelectEnabled}
        label="Select"
        name="Select recording clip"
        onSelectedChange={(selected) => {
          onToolChange(selected ? "select" : null);
        }}
        shortcut="V"
      >
        <MousePointer2 />
      </PreviewToolToggle>
      <PreviewToolToggle
        isDisabled={!isFrameEnabled}
        isSelected={tool === "canvas" && isFrameEnabled}
        label="Resize frame"
        name="Resize recording frame"
        onSelectedChange={(selected) => {
          onToolChange(selected ? "canvas" : null);
        }}
        shortcut="F"
      >
        <ScanSquare />
      </PreviewToolToggle>
      <PreviewToolToggle
        isDisabled={!isEnabled}
        isSelected={tool === "crop" && isEnabled}
        label="Crop"
        name="Crop recording clip"
        onSelectedChange={(selected) => {
          onToolChange(selected ? "crop" : null);
        }}
        shortcut="C"
      >
        <Crop />
      </PreviewToolToggle>
      <PreviewToolToggle
        isDisabled={!isRecenterEnabled}
        isSelected={tool === "recenter" && isRecenterEnabled}
        label="Recenter from current frame"
        name="Recenter recording from current frame"
        onSelectedChange={(selected) => {
          onToolChange(selected ? "recenter" : null);
        }}
        shortcut="R"
      >
        <CircleDotDashed />
      </PreviewToolToggle>
      <PreviewToolReset
        canvasLabel="Reset frame"
        isDisabled={!resetEnabled || !canReset}
        onReset={reset}
        tool={tool}
      />
    </>
  );
}
