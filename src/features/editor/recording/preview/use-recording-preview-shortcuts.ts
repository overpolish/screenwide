// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { Arrangement } from "../../annotations/annotation-order";
import { placeRecordingImage } from "../../images/image-api";
import { useIncomingImages } from "../../images/use-incoming-images";
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
  arrangeScenePane,
  canEditActiveTrack,
  canResizeActiveTrack,
  canvasTool,
  changeCanvasTool,
  hasAnnotations,
  hasCursorData,
  hasKeyboardData,
  hasSceneSelection,
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
  /** Move the selected pane through the order of the custom scene under the
   * playhead; unset where there is no such scene or pane. */
  arrangeScenePane: ((move: Arrangement) => void) | undefined;
  canEditActiveTrack: boolean;
  canResizeActiveTrack: boolean;
  canvasTool: RecordingCanvasTool;
  changeCanvasTool: ReturnType<
    typeof useRecordingPreviewCanvasTool
  >["changeCanvasTool"];
  /** Whether annotations can be drawn, which the annotation tools and an
   * incoming image are for. */
  hasAnnotations: boolean;
  hasCursorData: boolean;
  hasKeyboardData: boolean;
  /** Scenes chosen in their lane own Delete, so nothing under the pointer
   * goes with them. */
  hasSceneSelection: boolean;
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
    // A chosen annotation moves through the drawing order. Outside a custom
    // scene the video layers keep theirs, the camera drawn over the screen;
    // in one, the selected pane moves through the scene's order.
    onArrange: annotations.hasSelection
      ? (move) => {
          annotations.arrangeSelected(move);
        }
      : arrangeScenePane,
    onConfirm: isCropping ? leaveCropTool : undefined,
    onDelete:
      annotations.canDelete && !hasSceneSelection
        ? annotations.deleteTargeted
        : undefined,
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
    onTool:
      hasVisiblePanes && hasAnnotations
        ? (tool) => {
            toggleTool[tool]();
          }
        : undefined,
    ownsEscape: isCropping || annotations.hasSelection,
  });
  useIncomingImages(
    "recording",
    hasVisiblePanes && hasAnnotations ? placeRecordingImage : undefined,
    {
      // The image placed is chosen; a tool that shows it is put in hand,
      // unless the select tool or the image tool already is.
      onPlaced: () => {
        if (canvasTool !== "select" && canvasTool !== "image")
          changeCanvasTool("image");
      },
      settle: isPlaying ? player.pauseSettled : undefined,
    },
  );
}
