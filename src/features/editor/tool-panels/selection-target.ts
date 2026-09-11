// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingOutputSettings,
  resetScreenshotTransform,
  ScreenshotOutputSettings,
  screenshotWorkspaceItemOutput,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import {
  selectionPlacement,
  withSelectionPlacement,
} from "../selection-placement";
import { EditorArtifact, RecordingVideoTrackId } from "../types";

import { ToolPanelHandlers } from "./tool-panel-bridge";
import { ToolPanelSelection } from "./tool-panel-store";

/**
 * The one layer the selection panel acts on: what to show for it, the output
 * it is placed in, and the editor handler that commits a new placement.
 *
 * The editor already owns every one of those; this only says which of them the
 * current selection means, so the panel never has to know whether it is
 * looking at a recording track or a screenshot layer.
 */
export type EditorSelectionTarget = {
  apply: (next: ScreenshotOutputSettings) => void;
  selection: ToolPanelSelection;
  settings: ScreenshotOutputSettings;
  source: { height: number; width: number };
};

type SelectionTargetInputs = {
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

const target = ({
  apply,
  kind,
  label,
  settings,
  source,
}: Omit<EditorSelectionTarget, "selection"> & {
  kind: ToolPanelSelection["kind"];
  label: string;
}): EditorSelectionTarget => ({
  apply,
  selection: {
    ...selectionPlacement(settings),
    kind,
    label,
    sourceHeight: source.height,
    sourceWidth: source.width,
  },
  settings,
  source,
});

const recordingSelectionTarget = (
  inputs: SelectionTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "recording" }>,
): EditorSelectionTarget | null => {
  const track =
    inputs.selectedTrack === "primary" || inputs.selectedTrack === "camera"
      ? inputs.selectedTrack
      : null;
  const output = inputs.recordingOutput;
  if (!track || !output || !inputs.enabledVideoTracks.includes(track))
    return null;
  // A baked camera is placed by its overlay rather than by an output of its
  // own, so there is no placement here to show for it.
  if (track === "camera" && inputs.bakeCamera) return null;
  const source =
    track === "primary"
      ? { height: artifact.height, width: artifact.width }
      : artifact.camera;
  if (!source) return null;
  return target({
    apply: (next) => {
      inputs.onRecordingOutputChange?.(track, next);
    },
    kind: track,
    label: track === "primary" ? "Screen" : "Camera",
    settings: output[track],
    source,
  });
};

const screenshotSelectionTarget = (
  inputs: SelectionTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "screenshot" }>,
): EditorSelectionTarget | null => {
  const output = inputs.screenshotOutput;
  const index = artifact.items.findIndex(
    (item) => item.id === inputs.selectedScreenshotItemId,
  );
  if (!output || index < 0) return null;
  const item = artifact.items[index];
  // A composite carries no layer names, so a layer is known by where it sits
  // in the stack, counted from the bottom the way the layer actions count it.
  return target({
    apply: (next) => {
      inputs.onScreenshotOutputChange?.(next, item.id);
    },
    kind: "layer",
    label:
      artifact.items.length > 1 ? `Layer ${String(index + 1)}` : "Screenshot",
    settings: screenshotWorkspaceItemOutput(output, item.id),
    source: { height: item.height, width: item.width },
  });
};

export const editorSelectionTarget = (
  inputs: SelectionTargetInputs,
): EditorSelectionTarget | null => {
  const { artifact } = inputs;
  if (!artifact) return null;
  return artifact.kind === "recording"
    ? recordingSelectionTarget(inputs, artifact)
    : screenshotSelectionTarget(inputs, artifact);
};

/**
 * The selection panel's asks, answered against whatever is selected now.
 *
 * Both go back out through the workspace's own output handler, so a number
 * typed in the panel takes the identical path through the editor's state as
 * the drag that would have produced it.
 */
export const selectionPanelHandlers = (
  target: EditorSelectionTarget | null,
): Pick<
  ToolPanelHandlers,
  "onSelectionPlacementChange" | "onSelectionReset"
> => ({
  onSelectionPlacementChange: (placement) => {
    console.debug("[selection-panel] placement request", {
      before: target ? selectionPlacement(target.settings) : null,
      hasTarget: Boolean(target),
      placement,
    });
    if (!target) return;
    const next = withSelectionPlacement(
      target.settings,
      target.source,
      placement,
    );
    console.debug("[selection-panel] applying", selectionPlacement(next));
    target.apply(next);
  },
  // The reset the Select tool has always had: the layer back to the framing
  // its source arrived in, its crop kept.
  onSelectionReset: () => {
    if (!target) return;
    target.apply(resetScreenshotTransform(target.settings, target.source));
  },
});
