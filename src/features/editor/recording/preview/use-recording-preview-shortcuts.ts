// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import { useRecordingTimelineBlade } from "../../timeline/editing/use-recording-timeline-blade";
import { RecordingPreviewLayout } from "../../types";
import { useRecordingAnnotations } from "../annotations/use-recording-annotations";
import { useRecordingPreviewPlayer } from "../use-recording-preview-player";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import { useRecordingPreviewCanvasTool } from "./use-recording-preview-canvas-tool";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";

/** Every window shortcut the recording preview answers, and the transport
 * toggle the space bar drives. */
export function useRecordingPreviewShortcuts({
  annotations,
  canEditActiveTrack,
  canResizeActiveTrack,
  canvasTool,
  hasCursorData,
  hasKeyboardData,
  hasScenes,
  hasVisiblePanes,
  isCropping,
  layout,
  leaveCropTool,
  nudgeActiveTrack,
  player,
  step,
  toggleCursorPanel,
  toggleKeyboardPanel,
  toggleTool,
}: {
  annotations: ReturnType<typeof useRecordingAnnotations>;
  canEditActiveTrack: boolean;
  canResizeActiveTrack: boolean;
  canvasTool: RecordingCanvasTool;
  hasCursorData: boolean;
  hasKeyboardData: boolean;
  hasScenes: boolean;
  hasVisiblePanes: boolean;
  isCropping: boolean;
  layout: RecordingPreviewLayout | null;
  leaveCropTool: () => void;
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
    canvasTool === "select" && canEditActiveTrack && !annotations.hasSelection;
  useEditorWindowShortcuts({
    // A chosen annotation moves through the drawing order. The video layers
    // keep theirs: the camera is always drawn over the screen.
    onArrange: annotations.hasSelection
      ? (move) => {
          annotations.arrangeSelected(move);
        }
      : undefined,
    onConfirm: isCropping ? leaveCropTool : undefined,
    onDelete: annotations.canDelete ? annotations.deleteTargeted : undefined,
    onDeselect: annotations.hasSelection
      ? annotations.clearSelection
      : isCropping
        ? leaveCropTool
        : undefined,
    onMarqueeTool: hasVisiblePanes ? toggleTool.marquee : undefined,
    onNudge: canNudgeActiveTrack ? nudgeActiveTrack : undefined,
    onResizeCanvas: canResizeActiveTrack ? toggleTool.canvas : undefined,
    onSceneTool: hasScenes && hasVisiblePanes ? toggleTool.scene : undefined,
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
