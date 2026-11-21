// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { shownTool } from "../../choice/editor-choice";
import { useNativeCropRequest } from "../../preview/use-native-crop-request";
import { usePublishCropTool } from "../../tools/crop/crop-panel-handlers";
import {
  columnMode,
  columnTool,
  modeTool,
  recordingIsland,
  type RecordingPanelMode,
  type RecordingPicture,
} from "../recording-column";

import type { RecordingCanvasTool } from "./recording-canvas-tool";
import type { AnnotationKind } from "../../../annotations/types";
import type { EditorMode } from "../../toolbar/editor-modes";
import type { EditorTool } from "../../toolbar/editor-tool-column";
import type { RecordingTrackId, RecordingVideoTrackId } from "../../types";

/**
 * The recording's tool column, its modes in the title bar, and the island
 * they bring out: the pen, the crop or Select in hand, the mode open, and the
 * island for them, which `useRecordingIsland` shows. Frame is the picture's
 * tool, a drag of its own. The other modes are only an island, open over
 * Select, and put away by any of the picture's tools being taken up,
 * however it is. Screen and Camera are their track chosen with Select in
 * hand, however it was chosen, so choosing a layer opens its mode over any
 * other island-only one, Scene included. Select taken up never lets the
 * layer go: only Escape, a press on nothing and ⌘D do. The crop is the
 * chosen layer's, the screen's where none is, and put away it leaves the
 * layer chosen.
 */
export function useRecordingColumn({
  activeVideoTrack,
  annotations,
  canvasTool,
  changeCanvasTool,
  isHeld,
  onSelectedTrackChange,
  picture,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  annotations: {
    clearSelection: () => void;
    hasSelection: boolean;
    selectedIds: ReadonlySet<string>;
    selectedKind: AnnotationKind | null;
  };
  canvasTool: RecordingCanvasTool;
  changeCanvasTool: (tool: RecordingCanvasTool) => void;
  /** Whether ⌘ (Ctrl) holds Select over the tool. */
  isHeld: boolean;
  /** What the recording's own picture is of, which names its mode. */
  picture: RecordingPicture;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
}) {
  const [panel, setPanel] = useState<RecordingPanelMode | null>(null);
  if (panel !== null && (canvasTool !== "select" || activeVideoTrack !== null))
    setPanel(null);
  // The pen a click on an annotation calls for shows at once, though it is
  // taken up only as the press ends, so the column and the title bar never
  // pass through Select on the way to it.
  const shown = shownTool({
    isHeld,
    kind: annotations.selectedKind,
    tool: canvasTool,
  });
  // A track's mode is Select on the track with no annotations chosen:
  // annotations chosen with Select's band show their own island instead.
  const mode = columnMode({
    panel,
    picture,
    tool: shown,
    track: annotations.hasSelection ? null : activeVideoTrack,
  });
  const island = recordingIsland({
    annotationCount: annotations.selectedIds.size,
    mode,
    selectedKind: annotations.selectedKind,
    tool: canvasTool,
  });

  /** Select in hand with nothing chosen, a track included, and no mode
   * open. The choice is let go after Select is taken up, which would
   * otherwise bring back the annotations it held when it was put down
   * (`useSelectToolMemory`). */
  const backToSelect = () => {
    changeCanvasTool("select");
    annotations.clearSelection();
    onSelectedTrackChange?.(null);
    setPanel(null);
  };
  /** Select taken up, by its button, V, or the crop put away by Done,
   * Return, Escape or a press past its picture: the layer chosen stays
   * chosen, so its mode opens or stays open (user asked: Select letting the
   * camera go only to have it chosen again read as broken). The annotations
   * go, as a mode lets them go. */
  const selectTool = () => {
    changeCanvasTool("select");
    annotations.clearSelection();
  };
  /** The crop taken up on `track`, else on the track in hand, the screen's
   * where none is. */
  const takeUpCrop = (track?: RecordingVideoTrackId) => {
    const layer = track ?? activeVideoTrack ?? "primary";
    if (layer !== activeVideoTrack) onSelectedTrackChange?.(layer);
    if (canvasTool !== "crop") changeCanvasTool("crop");
  };
  /** Opens `next`, letting go of the annotations chosen, or puts the open
   * mode away for none, with nothing chosen at all. */
  const chooseMode = (next: EditorMode | null) => {
    if (next === null) {
      backToSelect();
      return;
    }
    const taken = modeTool(next, picture);
    annotations.clearSelection();
    // An island-only mode lets the video track go, which would open its own
    // mode over it; an audio track chosen stays.
    if (taken.track !== undefined && (taken.track ?? activeVideoTrack) !== null)
      onSelectedTrackChange?.(taken.track);
    setPanel(taken.panel);
    if (taken.tool !== canvasTool) changeCanvasTool(taken.tool);
  };
  /** A mode's key: opens it, or puts it away while it is open. */
  const toggleMode = (next: EditorMode) => {
    chooseMode(mode === next ? null : next);
  };
  const ownsEscape =
    canvasTool !== "select" || mode !== null || annotations.hasSelection;
  /** Takes up `next` from the column: Select, keeping the layer chosen, or
   * a pen. */
  const takeUp = (next: EditorTool) => {
    if (next === "select") selectTool();
    else changeCanvasTool(next);
  };
  // A crop asked for on the picture: the layer double-clicked, `0` the
  // screen's and `1` the camera's, taken up for its crop, or none, a press
  // past the picture being cropped, which puts the crop away.
  useNativeCropRequest((layerId) => {
    if (layerId !== null) takeUpCrop(layerId === 0 ? "primary" : "camera");
    else if (canvasTool === "crop") selectTool();
  });
  // The crop taken up from the layer's island, or put away by its Done.
  usePublishCropTool((isCropping) => {
    if (isCropping) takeUpCrop();
    else if (canvasTool === "crop") selectTool();
  });
  return {
    backToSelect,
    column: {
      isModeOpen: mode !== null,
      onToolChange: takeUp,
      tool: columnTool(shown),
    },
    /** The island the pen, the chosen annotations or the open mode bring
     * out, and the tool that brought it. */
    island,
    /** The mode open, and the title bar's way to open one or put it away. */
    mode,
    onModeChange: chooseMode,
    /** A chosen overlay opens its mode, and leaves it open if it is. */
    openMode: (next: EditorMode) => {
      if (mode !== next) chooseMode(next);
    },
    ownsEscape,
    /** Select taken up by its key, or the crop put away by Return or
     * Escape: the layer stays chosen. */
    selectTool,
    /** K: the crop taken up, or put away while it is in hand. */
    toggleCrop: () => {
      if (canvasTool === "crop") selectTool();
      else takeUpCrop();
    },
    toggleMode,
  };
}
