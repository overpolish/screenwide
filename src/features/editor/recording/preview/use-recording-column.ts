// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { shownTool } from "../../choice/editor-choice";
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
 * they bring out: the pen, Crop or Select in hand, the mode open, and the
 * island for them, which `useRecordingIsland` shows. A mode with a drag of
 * its own (Frame, Scene) is the picture's tool. The rest are only an
 * island, open over Select, and put away by any of the picture's tools
 * being taken up, however it is. Screen and Camera are their track chosen
 * with Select in hand, however it was chosen, so choosing a layer opens its
 * mode over any other island-only one. Crop crops the track in hand, the
 * screen's where none is.
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
  /** Takes up `next` from the column: Select with nothing chosen, or a pen
   * or Crop, which takes the screen's track in hand where none is. */
  const takeUp = (next: EditorTool) => {
    if (next === "select") {
      backToSelect();
      return;
    }
    if (next === "crop" && activeVideoTrack === null)
      onSelectedTrackChange?.("primary");
    changeCanvasTool(next);
  };
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
    /** C: Crop taken up, or put down while it is in hand. */
    toggleCrop: () => {
      takeUp(canvasTool === "crop" ? "select" : "crop");
    },
    toggleMode,
  };
}
