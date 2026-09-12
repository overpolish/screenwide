// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { refreshRecenterInset } from "../recenter-inset-channel";
import {
  commitScreenshotCrop,
  resetCommittedScreenshotCrop,
} from "../screenshot-crop";
import {
  fullSourceRect,
  sourceRect,
  SourceRect,
} from "../screenshot-geometry";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  screenshotWorkspaceItemOutput,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import {
  screenshotSourceCrop,
  withScreenshotSourceCrop,
} from "../screenshot-output-settings";
import { EditorArtifact, EditorKind, RecordingVideoTrackId } from "../types";

import { ToolPanelHandlers } from "./tool-panel-bridge";
import { ToolPanelCrop } from "./tool-panel-store";

/**
 * The one source rectangle the crop panel acts on: what to show for it, and
 * the editor path that commits a new one.
 *
 * Both are the paths a crop drag already takes - the same
 * `commitScreenshotCrop` a released handle goes through - so a size typed in
 * the panel rebases the outer inset frame exactly as the gesture's would, and
 * the panel never has to know whether it is looking at a recording track or a
 * screenshot layer.
 */
export type EditorCropTarget = {
  /** Commit a crop of this size in source pixels, kept where it sits. */
  apply: (size: { height: number; width: number }) => void;
  /** Back to the whole source, the committed crop taken away. */
  reset: () => void;
  /** The crop as the panel shows it, in source pixels. */
  snapshot: ToolPanelCrop;
};

type CropTargetInputs = {
  artifact: EditorArtifact | null;
  bakeCamera: boolean;
  enabledVideoTracks: RecordingVideoTrackId[];
  recordingOutput: RecordingOutputSettings | null | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
  onRecordingOutputChange?: (
    track: RecordingVideoTrackId,
    next: ScreenshotOutputSettings,
  ) => void;
  onScreenshotOutputChange?: (
    next: ScreenshotOutputSettings,
    itemId: number,
  ) => void;
};

const clamp = (value: number, minimum: number, maximum: number) =>
  Math.min(maximum, Math.max(minimum, value));

/**
 * A crop of the asked-for size, around the middle of the one in hand.
 *
 * Typing a size says how much of the source to keep, not which part of it: the
 * rectangle grows and shrinks about its own centre, and slides back inside the
 * source rather than shrinking when that centre is near an edge. A size past
 * the whole source is the whole source; a size below one source pixel is one.
 */
export const centredSourceCrop = (
  current: SourceRect,
  size: { height: number; width: number },
  source: { height: number; width: number },
): SourceRect => {
  const width = clamp(size.width / source.width, 1 / source.width, 1);
  const height = clamp(size.height / source.height, 1 / source.height, 1);
  return sourceRect({
    height,
    width,
    x: clamp(current.x + current.width / 2 - width / 2, 0, 1 - width),
    y: clamp(current.y + current.height / 2 - height / 2, 0, 1 - height),
  });
};

const target = ({
  applyOutput,
  settings,
  source,
  workspace,
}: {
  /** Commit a whole output, the path the workspace's own gesture takes. */
  applyOutput: (next: ScreenshotOutputSettings) => void;
  settings: ScreenshotOutputSettings;
  source: { height: number; width: number };
  /** The workspace whose recenter inset to re-read, or null where a crop
   * there never carries one. */
  workspace: EditorKind | null;
}): EditorCropTarget => {
  const crop = screenshotSourceCrop(settings);
  const refresh = (next: SourceRect) => {
    if (workspace) refreshRecenterInset(workspace, next);
  };
  return {
    apply: (size) => {
      const next = centredSourceCrop(crop, size, source);
      applyOutput(
        commitScreenshotCrop(
          settings,
          withScreenshotSourceCrop(settings, next),
          source,
        ),
      );
      refresh(next);
    },
    reset: () => {
      applyOutput(resetCommittedScreenshotCrop(settings, source));
      refresh(fullSourceRect());
    },
    snapshot: {
      height: Math.round(crop.height * source.height),
      sourceHeight: source.height,
      sourceWidth: source.width,
      width: Math.round(crop.width * source.width),
    },
  };
};

const recordingCropTarget = (
  inputs: CropTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "recording" }>,
): EditorCropTarget | null => {
  const output = inputs.recordingOutput;
  if (!output) return null;
  const bakedCamera =
    inputs.bakeCamera &&
    inputs.enabledVideoTracks.includes("primary") &&
    inputs.enabledVideoTracks.includes("camera");
  // The same track a frame edit lands on: a separate camera is cropped in its
  // own output, and a baked one has none of its own to crop.
  const track: RecordingVideoTrackId =
    inputs.selectedTrack === "camera" && !bakedCamera && artifact.camera
      ? "camera"
      : "primary";
  const source =
    track === "camera" && artifact.camera
      ? { height: artifact.camera.height, width: artifact.camera.width }
      : { height: artifact.height, width: artifact.width };
  return target({
    applyOutput: (next) => {
      inputs.onRecordingOutputChange?.(track, next);
    },
    settings: output[track],
    source,
    // Only the screen track carries a recenter inset, so only its crop has
    // one to look at again.
    workspace: track === "primary" ? "recording" : null,
  });
};

const screenshotCropTarget = (
  inputs: CropTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "screenshot" }>,
): EditorCropTarget | null => {
  const output = inputs.screenshotOutput;
  if (!output || artifact.items.length === 0) return null;
  // The layer in hand, and the newest one when nothing is selected: the very
  // layer the Crop toolbar button selects before it takes the tool up.
  const item =
    artifact.items.find(
      (candidate) => candidate.id === inputs.selectedScreenshotItemId,
    ) ?? artifact.items[artifact.items.length - 1];
  return target({
    applyOutput: (next) => {
      inputs.onScreenshotOutputChange?.(next, item.id);
    },
    settings: screenshotWorkspaceItemOutput(output, item.id),
    source: { height: item.height, width: item.width },
    workspace: "screenshot",
  });
};

export const editorCropTarget = (
  inputs: CropTargetInputs,
): EditorCropTarget | null => {
  const { artifact } = inputs;
  if (!artifact) return null;
  return artifact.kind === "recording"
    ? recordingCropTarget(inputs, artifact)
    : screenshotCropTarget(inputs, artifact);
};

/**
 * The crop panel's asks, answered against the workspace's own source.
 *
 * A field left blank keeps the size it already had: only the number that was
 * typed moves, so a linked pair sends both and a single field sends one.
 */
export const cropPanelHandlers = (
  target: EditorCropTarget | null,
): Pick<ToolPanelHandlers, "onCropReset" | "onCropSizeChange"> => ({
  onCropReset: () => {
    target?.reset();
  },
  onCropSizeChange: (size) => {
    if (!target) return;
    target.apply({
      height: size.height ?? target.snapshot.height,
      width: size.width ?? target.snapshot.width,
    });
  },
});
