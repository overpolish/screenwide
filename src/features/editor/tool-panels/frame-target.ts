// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Background } from "../../../components/shared/background-picker/background";
import {
  applyBackgroundToOutput,
  backgroundFromOutput,
} from "../background-adapters";
import {
  RecordingOutputSettings,
  resizeScreenshotCanvas,
  resizeScreenshotWorkspaceCanvas,
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import { EditorArtifact, RecordingVideoTrackId } from "../types";

import { ToolPanelHandlers } from "./tool-panel-bridge";
import { ToolPanelFrame } from "./tool-panel-store";

/**
 * The one canvas the frame panel acts on: what to show for it, and the two
 * editor paths that change it.
 *
 * Both are the paths a frame drag already takes, so a size typed in the panel
 * lands in the workspace's state exactly as the gesture's would, and the panel
 * never has to know whether it is looking at a recording or a screenshot.
 */
export type EditorFrameTarget = {
  /** Commit a canvas of this size, leaving what is in it where it sits. */
  apply: (size: { height: number; width: number }) => void;
  /** Fill the canvas with this background, leaving the other kinds' values
   * where they were so a swap back finds them unchanged. */
  applyBackground: (background: Background) => void;
  /** The canvas's background, as the picker shows it. */
  background: Background;
  /** Back to the source size, with what is in it refitted to the new canvas. */
  reset: () => void;
  /** The canvas as the panel shows it. */
  snapshot: ToolPanelFrame;
};

type FrameTargetInputs = {
  artifact: EditorArtifact | null;
  bakeCamera: boolean;
  enabledVideoTracks: RecordingVideoTrackId[];
  recordingOutput: RecordingOutputSettings | null | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onRecordingOutputChange?: (
    track: RecordingVideoTrackId,
    next: ScreenshotOutputSettings,
  ) => void;
};

const target = ({
  apply,
  applyOutput,
  reset,
  settings,
  source,
}: Pick<EditorFrameTarget, "apply" | "reset"> & {
  /** Commit a whole canvas, the path the workspace's own controls take. */
  applyOutput: (next: ScreenshotOutputSettings) => void;
  settings: ScreenshotOutputSettings;
  source: { height: number; width: number };
}): EditorFrameTarget => ({
  apply,
  applyBackground: (background) => {
    applyOutput(applyBackgroundToOutput(settings, background));
  },
  background: backgroundFromOutput(settings),
  reset,
  snapshot: {
    ...screenshotOutputDimensions(settings),
    sourceHeight: source.height,
    sourceWidth: source.width,
  },
});

const recordingFrameTarget = (
  inputs: FrameTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "recording" }>,
): EditorFrameTarget | null => {
  const output = inputs.recordingOutput;
  if (!output) return null;
  const bakedCamera =
    inputs.bakeCamera &&
    inputs.enabledVideoTracks.includes("primary") &&
    inputs.enabledVideoTracks.includes("camera");
  // The frame is the selected track's own output canvas. A separate camera
  // has one of its own; a baked camera lives in the screen's, so the screen
  // canvas is the frame whenever the camera is not separate.
  const track: RecordingVideoTrackId =
    inputs.selectedTrack === "camera" && !bakedCamera && artifact.camera
      ? "camera"
      : "primary";
  const source =
    track === "camera" && artifact.camera
      ? { height: artifact.camera.height, width: artifact.camera.width }
      : { height: artifact.height, width: artifact.width };
  const settings = output[track];
  return target({
    // The inspector's own canvas resize: only the canvas changes size. What
    // is placed in it, a baked camera overlay included, is measured in output
    // pixels and so keeps the place in the picture it was put in.
    apply: ({ height, width }) => {
      inputs.onRecordingOutputChange?.(
        track,
        resizeScreenshotCanvas({ height, settings, width }),
      );
    },
    applyOutput: (next) => {
      inputs.onRecordingOutputChange?.(track, next);
    },
    // Resetting the frame is resizing it back to the source size and no
    // more: the layer keeps the transform the Select tool gave it, as it
    // would through a typed size.
    reset: () => {
      inputs.onRecordingOutputChange?.(
        track,
        resizeScreenshotCanvas({
          height: source.height,
          settings,
          width: source.width,
        }),
      );
    },
    settings,
    source,
  });
};

const screenshotFrameTarget = (
  inputs: FrameTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "screenshot" }>,
): EditorFrameTarget | null => {
  const output = inputs.screenshotOutput;
  if (!output || artifact.items.length === 0) return null;
  // The canvas is sized against the layer in hand, and against the bottom one
  // when nothing is selected: that is the capture the workspace opened at.
  const item =
    artifact.items.find(
      (candidate) => candidate.id === inputs.selectedScreenshotItemId,
    ) ?? artifact.items[0];
  const source = { height: item.height, width: item.width };
  return target({
    apply: ({ height, width }) => {
      inputs.onCanvasResize?.(
        resizeScreenshotWorkspaceCanvas({ height, settings: output, width }),
      );
    },
    // The background is a property of the workspace canvas: every layer reads
    // it from there, so it is written there and nowhere else.
    applyOutput: (next) => {
      inputs.onCanvasResize?.({ ...output, ...next });
    },
    // Resetting the frame is resizing it back to the source size and no
    // more: every layer keeps the transform the Select tool gave it, as it
    // would through a typed size.
    reset: () => {
      inputs.onCanvasResize?.(
        resizeScreenshotWorkspaceCanvas({
          height: source.height,
          settings: output,
          width: source.width,
        }),
      );
    },
    settings: output,
    source,
  });
};

export const editorFrameTarget = (
  inputs: FrameTargetInputs,
): EditorFrameTarget | null => {
  const { artifact } = inputs;
  if (!artifact) return null;
  return artifact.kind === "recording"
    ? recordingFrameTarget(inputs, artifact)
    : screenshotFrameTarget(inputs, artifact);
};

/**
 * The frame panel's asks, answered against the workspace's canvas.
 *
 * A field left blank keeps the size it already had: only the number that was
 * typed moves, so a linked pair sends both and a single field sends one.
 */
export const framePanelHandlers = (
  target: EditorFrameTarget | null,
): Pick<
  ToolPanelHandlers,
  "onBackgroundChange" | "onFrameReset" | "onFrameSizeChange"
> => ({
  onBackgroundChange: (background) => {
    target?.applyBackground(background);
  },
  onFrameReset: () => {
    target?.reset();
  },
  onFrameSizeChange: (size) => {
    if (!target) return;
    target.apply({
      height: size.height ?? target.snapshot.height,
      width: size.width ?? target.snapshot.width,
    });
  },
});
