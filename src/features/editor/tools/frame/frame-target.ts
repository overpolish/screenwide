// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  canvasScale,
  scaledCameraOverlay,
  scaleScreenshotCanvas,
  scaleScreenshotWorkspaceCanvas,
} from "../../screenshot/screenshot-canvas-scale";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
  ScreenshotWorkspaceOutputSettings,
} from "../../screenshot/screenshot-output";
import {
  CameraOverlaySettings,
  EditorArtifact,
  RecordingVideoTrackId,
} from "../../types";
import { ToolPanelHandlers } from "../tool-panel-handlers";
import { ToolPanelFrame } from "../tool-panel-store";

import { Background } from "./background/background";
import {
  applyBackgroundToOutput,
  backgroundFromOutput,
} from "./background-adapters";

/**
 * The one canvas the frame panel acts on: what to show for it, and the two
 * editor paths that change it.
 *
 * Both are the paths a frame drag already takes, so a size typed in the panel
 * lands in the workspace's state exactly as the gesture's would, and the panel
 * never has to know whether it is looking at a recording or a screenshot.
 */
export type EditorFrameTarget = {
  /** Commit a canvas of this size, what is in it scaled with it so the
   * composition keeps its proportions. */
  apply: (size: { height: number; width: number }) => void;
  /** Fill the canvas with this background, leaving the other kinds' values
   * where they were so a swap back finds them unchanged. */
  applyBackground: (background: Background) => void;
  /** Round the canvas corners by this share of its shorter side. */
  applyRadius: (radius: number) => void;
  /** The canvas's background, as the picker shows it. */
  background: Background;
  /** Back to the source size, what is in it scaled with it. */
  reset: () => void;
  /** The canvas as the panel shows it. */
  snapshot: ToolPanelFrame;
};

type FrameTargetInputs = {
  artifact: EditorArtifact | null;
  bakeCamera: boolean;
  /** The camera drawn into the screen's canvas, which a new size scales as
   * it does the screen. */
  cameraOverlay: CameraOverlaySettings;
  /** The video tracks the workspace draws, which leave out a camera saved as
   * a separate file. */
  composedVideoTracks: RecordingVideoTrackId[];
  recordingOutput: RecordingOutputSettings | null | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
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
  applyRadius: (radius) => {
    applyOutput({ ...settings, backgroundRadiusPercent: radius });
  },
  background: backgroundFromOutput(settings),
  reset,
  snapshot: {
    ...screenshotOutputDimensions(settings),
    radius: settings.backgroundRadiusPercent,
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
    inputs.composedVideoTracks.includes("primary") &&
    inputs.composedVideoTracks.includes("camera");
  // The frame is the selected track's own output canvas. A camera drawn as
  // its own pane has one of its own; a baked camera lives in the screen's, so
  // the screen canvas is the frame whenever the camera is not its own pane.
  const track: RecordingVideoTrackId =
    inputs.selectedTrack === "camera" &&
    inputs.composedVideoTracks.includes("camera") &&
    !bakedCamera &&
    artifact.camera
      ? "camera"
      : "primary";
  const source =
    track === "camera" && artifact.camera
      ? { height: artifact.camera.height, width: artifact.camera.width }
      : { height: artifact.height, width: artifact.width };
  const settings = output[track];
  // The canvas and everything in it scale together: the layer, and a camera
  // drawn into the screen's canvas, measured in its pixels. Both change in
  // one commit, so they are one undo step.
  const resize = (size: { height: number; width: number }) => {
    inputs.onRecordingOutputChange?.(
      track,
      scaleScreenshotCanvas({ ...size, settings }),
    );
    if (bakedCamera && track === "primary")
      inputs.onCameraOverlayChange?.(
        scaledCameraOverlay(inputs.cameraOverlay, canvasScale(settings, size)),
      );
  };
  return target({
    apply: resize,
    applyOutput: (next) => {
      inputs.onRecordingOutputChange?.(track, next);
    },
    reset: () => {
      resize(source);
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
        scaleScreenshotWorkspaceCanvas({ height, settings: output, width }),
      );
    },
    // The background is a property of the workspace canvas: every layer reads
    // it from there, so it is written there and nowhere else.
    applyOutput: (next) => {
      inputs.onCanvasResize?.({ ...output, ...next });
    },
    reset: () => {
      inputs.onCanvasResize?.(
        scaleScreenshotWorkspaceCanvas({
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
  | "onBackgroundChange"
  | "onFrameRadiusChange"
  | "onFrameReset"
  | "onFrameSizeChange"
> => ({
  onBackgroundChange: (background) => {
    target?.applyBackground(background);
  },
  onFrameRadiusChange: (radius) => {
    target?.applyRadius(radius);
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
