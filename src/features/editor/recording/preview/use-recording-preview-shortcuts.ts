// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { Arrangement } from "../../annotations/annotation-order";
import { placeRecordingImage } from "../../images/image-api";
import { useIncomingImages } from "../../images/use-incoming-images";
import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import { useRecordingTimelineBlade } from "../../timeline/editing/use-recording-timeline-blade";
import { EditorMode } from "../../toolbar/editor-modes";
import { RecordingPreviewLayout } from "../../types";
import { useRecordingAnnotations } from "../annotations/use-recording-annotations";
import { useRecordingPreviewPlayer } from "../use-recording-preview-player";

import { RecordingCanvasTool } from "./recording-canvas-tool";
import { useRecordingPreviewCanvasTool } from "./use-recording-preview-canvas-tool";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";

/** Every window shortcut the recording preview answers, and the transport
 * toggle the space bar drives. A mode's key opens it as its button in the
 * tool column does, and Escape puts Select back with nothing chosen. */
export function useRecordingPreviewShortcuts({
  annotations,
  arrangeScenePane,
  backToSelect,
  canEditActiveTrack,
  canResizeActiveTrack,
  canvasTool,
  changeCanvasTool,
  hasAnnotations,
  hasSceneSelection,
  hasVisiblePanes,
  isCropping,
  layout,
  nudgeActiveTrack,
  offered,
  ownsEscape,
  player,
  step,
  toggleCrop,
  toggleMode,
  toggleTool,
}: {
  annotations: ReturnType<typeof useRecordingAnnotations>;
  /** Move the selected pane through the order of the custom scene under the
   * playhead; unset where there is no such scene or pane. */
  arrangeScenePane: ((move: Arrangement) => void) | undefined;
  /** Puts Select back in hand with nothing chosen and no mode open. */
  backToSelect: () => void;
  canEditActiveTrack: boolean;
  canResizeActiveTrack: boolean;
  canvasTool: RecordingCanvasTool;
  changeCanvasTool: ReturnType<
    typeof useRecordingPreviewCanvasTool
  >["changeCanvasTool"];
  /** Whether annotations can be drawn, which the annotation tools and an
   * incoming image are for. */
  hasAnnotations: boolean;
  /** Scenes chosen in their lane own Delete, so nothing under the pointer
   * goes with them. */
  hasSceneSelection: boolean;
  hasVisiblePanes: boolean;
  isCropping: boolean;
  layout: RecordingPreviewLayout | null;
  nudgeActiveTrack: ReturnType<
    typeof useRecordingPreviewSelection
  >["nudgeActiveTrack"];
  /** The modes the title bar offers; the key of any other does nothing. */
  offered: ReadonlySet<EditorMode>;
  /** Whether anything is in hand for Escape to put down. */
  ownsEscape: boolean;
  player: ReturnType<typeof useRecordingPreviewPlayer>;
  step: ReturnType<typeof useRecordingTimelineBlade>["step"];
  /** C: the Crop tool taken up or put down. */
  toggleCrop: () => void;
  toggleMode: (mode: EditorMode) => void;
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
  const mode = (next: EditorMode) => () => {
    toggleMode(next);
  };
  useEditorWindowShortcuts({
    // A chosen annotation moves through the drawing order. Outside a custom
    // scene the video layers keep theirs, the camera drawn over the screen;
    // in one, the selected pane moves through the scene's order.
    onArrange: annotations.hasSelection
      ? (move) => {
          annotations.arrangeSelected(move);
        }
      : arrangeScenePane,
    onConfirm: isCropping ? backToSelect : undefined,
    onDelete:
      annotations.canDelete && !hasSceneSelection
        ? annotations.deleteTargeted
        : undefined,
    onDeselect: ownsEscape ? backToSelect : undefined,
    // ⌘D lets go of everything chosen, whatever owns Escape: the
    // annotations, the track and the mode here, and with the mode the
    // scenes; the strip and the lanes let go of their own.
    onDeselectAll: backToSelect,
    onMarqueeTool: hasVisiblePanes ? toggleTool.marquee : undefined,
    onNudge: canNudgeActiveTrack ? nudgeActiveTrack : undefined,
    onResizeCanvas:
      canResizeActiveTrack && offered.has("frame") ? mode("frame") : undefined,
    onSceneTool:
      offered.has("scene") && hasVisiblePanes ? mode("scene") : undefined,
    onSelectTool: hasVisiblePanes ? backToSelect : undefined,
    onStep: !canNudgeActiveTrack && layout ? step : undefined,
    onToggleCaptionsPanel:
      offered.has("captions") && hasVisiblePanes ? mode("captions") : undefined,
    onToggleCrop: hasVisiblePanes ? toggleCrop : undefined,
    onToggleCursorPanel: offered.has("cursor") ? mode("cursor") : undefined,
    onToggleKeyboardPanel: offered.has("keyboard")
      ? mode("keyboard")
      : undefined,
    onTogglePlayback: layout ? togglePlayback : undefined,
    onToggleTranscriptPanel: offered.has("transcript")
      ? mode("transcript")
      : undefined,
    onTool:
      hasVisiblePanes && hasAnnotations
        ? (tool) => {
            toggleTool[tool]();
          }
        : undefined,
    ownsEscape,
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
