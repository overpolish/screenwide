// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { RecordingPreviewLayout } from "../types";
import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";
import { useRecordingAnnotations } from "../use-recording-annotations";
import { useRecordingPreviewPlayer } from "../use-recording-preview-player";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import { useRecordingPreviewCanvasTool } from "./use-recording-preview-canvas-tool";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { useRecordingTimelineBlade } from "./use-recording-timeline-blade";

/** Every window shortcut the recording preview answers, and the transport
 * toggle the space bar drives. */
export function useRecordingPreviewShortcuts({
  annotations,
  canMoveActiveVideoTrack,
  canResizeActiveTrack,
  canvasTool,
  hasCursorData,
  hasKeyboardData,
  hasVisiblePanes,
  isCropping,
  layout,
  leaveCropTool,
  moveActiveVideoTrackBackward,
  moveActiveVideoTrackForward,
  nudgeActiveTrack,
  player,
  step,
  toggleCursorPanel,
  toggleKeyboardPanel,
  toggleTool,
}: {
  annotations: ReturnType<typeof useRecordingAnnotations>;
  canMoveActiveVideoTrack: boolean;
  canResizeActiveTrack: boolean;
  canvasTool: RecordingCanvasTool;
  hasCursorData: boolean;
  hasKeyboardData: boolean;
  hasVisiblePanes: boolean;
  isCropping: boolean;
  layout: RecordingPreviewLayout | null;
  leaveCropTool: () => void;
  moveActiveVideoTrackBackward: () => void;
  moveActiveVideoTrackForward: () => void;
  nudgeActiveTrack: ReturnType<
    typeof useRecordingPreviewSelection
  >["nudgeActiveTrack"];
  player: ReturnType<typeof useRecordingPreviewPlayer>;
  step: ReturnType<typeof useRecordingTimelineBlade>["step"];
  toggleCursorPanel: () => void;
  toggleKeyboardPanel: () => void;
  toggleTool: ReturnType<typeof useRecordingPreviewCanvasTool>["toggleTool"];
}) {
  const isPlaying = player.isPlaying;
  const pause = player.pause;
  const play = player.play;
  const togglePlayback = useCallback(() => {
    if (isPlaying) pause();
    else play();
  }, [isPlaying, pause, play]);
  const canNudgeActiveTrack =
    canvasTool === "select" &&
    canMoveActiveVideoTrack &&
    !annotations.hasSelection;
  useEditorWindowShortcuts({
    onConfirm: isCropping ? leaveCropTool : undefined,
    onDelete: annotations.canDelete ? annotations.deleteTargeted : undefined,
    onDeselect: annotations.hasSelection
      ? annotations.clearSelection
      : isCropping
        ? leaveCropTool
        : undefined,
    onMoveBackward: canMoveActiveVideoTrack
      ? moveActiveVideoTrackBackward
      : undefined,
    onMoveForward: canMoveActiveVideoTrack
      ? moveActiveVideoTrackForward
      : undefined,
    onNudge: canNudgeActiveTrack ? nudgeActiveTrack : undefined,
    onResizeCanvas: canResizeActiveTrack ? toggleTool.canvas : undefined,
    onSelectTool: hasVisiblePanes ? toggleTool.select : undefined,
    onStep: !canNudgeActiveTrack && layout ? step : undefined,
    onToggleCrop: hasVisiblePanes ? toggleTool.crop : undefined,
    onToggleCursorPanel: hasCursorData ? toggleCursorPanel : undefined,
    onToggleKeyboardPanel: hasKeyboardData ? toggleKeyboardPanel : undefined,
    onTogglePlayback: layout ? togglePlayback : undefined,
    onTool: hasVisiblePanes
      ? (tool) => {
          toggleTool[tool]();
        }
      : undefined,
    ownsEscape: isCropping || annotations.hasSelection,
  });
}
