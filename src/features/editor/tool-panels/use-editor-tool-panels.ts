// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import {
  CursorEffectSettings,
  EditorArtifact,
  EditorKind,
  RecordingVideoTrackId,
} from "../types";

import { editorFrameTarget, framePanelHandlers } from "./frame-target";
import {
  editorSelectionTarget,
  selectionPanelHandlers,
} from "./selection-target";
import { useToolPanelBridge } from "./tool-panel-bridge";

type EditorToolPanelInputs = {
  artifact: EditorArtifact | null;
  bakeCamera: boolean;
  cursorEffects: CursorEffectSettings;
  enabledVideoTracks: RecordingVideoTrackId[];
  isSaving: boolean;
  recordingOutput: RecordingOutputSettings | null | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
  workspace: EditorKind;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onRecordingOutputChange?: (
    track: RecordingVideoTrackId,
    next: ScreenshotOutputSettings,
  ) => void;
  onScreenshotOutputChange?: (
    next: ScreenshotOutputSettings,
    itemId: number,
  ) => void;
};

/**
 * What the editor shows the tool panels, and what it lets them change.
 *
 * Each panel acts on one thing the editor already owns - the selected layer,
 * the output canvas - through the very handler the preview's own gesture uses,
 * so a number typed in a panel and a drag on the picture are the same edit.
 */
export function useEditorToolPanels({
  artifact,
  bakeCamera,
  cursorEffects,
  enabledVideoTracks,
  isSaving,
  onCanvasResize,
  onCursorEffectsChange,
  onRecordingOutputChange,
  onScreenshotOutputChange,
  recordingOutput,
  screenshotOutput,
  selectedScreenshotItemId,
  selectedTrack,
  workspace,
}: EditorToolPanelInputs) {
  // Which layer the selection panel is placing, and the handler that commits
  // a new placement for it. Both workspaces already own one; this only says
  // which of them the current selection means.
  const selectionTarget = editorSelectionTarget({
    artifact,
    bakeCamera,
    enabledVideoTracks,
    onRecordingOutputChange,
    onScreenshotOutputChange,
    recordingOutput,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
  });
  // The output canvas the frame panel sizes, and the workspace's own resize
  // path for it: the recording's primary output, or the screenshot canvas.
  const frameTarget = editorFrameTarget({
    artifact,
    bakeCamera,
    enabledVideoTracks,
    onCanvasResize,
    onRecordingOutputChange,
    recordingOutput,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
  });
  useToolPanelBridge(
    workspace,
    {
      cursorEffects,
      frame: frameTarget?.snapshot ?? null,
      hasCursorData: artifact?.kind === "recording" && artifact.hasCursorData,
      isSaving,
      selection: selectionTarget?.selection ?? null,
    },
    {
      onCursorEffectsChange,
      ...framePanelHandlers(frameTarget),
      ...selectionPanelHandlers(selectionTarget),
    },
  );
}
