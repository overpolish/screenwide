// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject } from "react";

import { t } from "../../../../i18n/i18n";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
} from "../../screenshot/screenshot-output";
import {
  RecordingPreviewLayout,
  RecordingPreviewPane,
  RecordingVideoTrackId,
} from "../../types";

import { NativeRecordingWorkspace } from "./native-recording-workspace-viewport";
import { RECORDING_PREVIEW_PANE_GAP } from "./recording-preview-layout";

type Entry = {
  canvasRef: RefObject<HTMLCanvasElement | null>;
  pane: RecordingPreviewPane;
  trackId: RecordingVideoTrackId;
};

/**
 * The baked composition is a single native pane: the camera overlay is drawn
 * into the primary output by the compositor, so the workspace only needs one
 * marker canvas whose bounds the native surface mirrors.
 */
export function bakedCameraWorkspace(
  outputSettings: ScreenshotOutputSettings,
  screenCanvasRef: RefObject<HTMLCanvasElement | null>,
): NativeRecordingWorkspace {
  const output = screenshotOutputDimensions(outputSettings);
  return {
    ariaLabel: t("editor-recording-preview"),
    panes: [
      {
        height: output.height,
        index: 0,
        label: t("editor-recording-composed-preview"),
        ref: screenCanvasRef,
        width: output.width,
        x: 0,
        y: 0,
      },
    ],
    workspaceHeight: output.height,
    workspaceWidth: output.width,
  };
}

/** The panes on screen side by side, each sized by its own output. */
export function recordingOutputWorkspace(
  entries: Entry[],
  outputs: RecordingOutputSettings,
): NativeRecordingWorkspace {
  const dimensions = entries.map(({ trackId }) =>
    screenshotOutputDimensions(outputs[trackId]),
  );
  const height = dimensions.reduce(
    (maximum, size) => Math.max(maximum, size.height),
    0,
  );
  const width = Math.max(
    1,
    dimensions.reduce((total, size) => total + size.width, 0) +
      Math.max(0, entries.length - 1) * RECORDING_PREVIEW_PANE_GAP,
  );
  let x = 0;
  const panes = entries.map((entry, index) => {
    const size = dimensions[index];
    const pane = {
      height: size.height,
      index: entry.trackId === "primary" ? 0 : 1,
      label:
        entry.pane.kind === "camera"
          ? t("editor-recording-camera-preview")
          : t("editor-recording-screen-preview"),
      ref: entry.canvasRef,
      width: size.width,
      x,
      y: (height - size.height) / 2,
    };
    x += size.width + RECORDING_PREVIEW_PANE_GAP;
    return pane;
  });
  return {
    ariaLabel: t("editor-recording-preview"),
    panes,
    workspaceHeight: height,
    workspaceWidth: width,
  };
}

/** The panes as the player laid them out, before any output is known. */
export function recordingLayoutWorkspace(
  layout: RecordingPreviewLayout,
  canvasRefs: RefObject<HTMLCanvasElement | null>[],
): NativeRecordingWorkspace {
  return {
    ariaLabel: t("editor-recording-preview"),
    panes: layout.panes.map((pane, index) => ({
      height: pane.height,
      index,
      label:
        pane.kind === "camera"
          ? t("editor-recording-camera-preview")
          : t("editor-recording-screen-preview"),
      ref: canvasRefs[index],
      width: pane.width,
      x: pane.x,
      y: pane.y,
    })),
    workspaceHeight: layout.height,
    workspaceWidth: layout.width,
  };
}
