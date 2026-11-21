// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { drawingToolKind } from "../tools/tool-registry";

import type { AnnotationKind } from "../../annotations/types";
import type { EditorIslandKind } from "../island/host/editor-island-kind";
import type { EditorMode } from "../toolbar/editor-modes";
import type { EditorTool } from "../toolbar/editor-tool-column";
import type { RecordingVideoTrackId } from "../types";
import type { RecordingCanvasTool } from "./preview/recording-canvas-tool";

// The recording's tools as the tool column and the title bar show them,
// and the island each brings out: the twin of `screenshot-column.ts`. The
// picture's own tools keep their names (`canvas` drags the frame, `crop`
// the crop of the track in hand); the title bar names the frame's by its
// mode. The crop is the chosen layer's, taken up from its island, its key or
// a double-click on it, so the column shows Select while it is in hand. The
// modes that work on the whole recording without a drag of their own
// (Scene, Cursor, Keyboard, Captions, Transcript, Sound) are only an island,
// so the picture keeps Select under them. Screen and Camera are Select on
// their track, however it was chosen, which Select then moves and sizes.

/** A mode that is only an island, with Select kept in hand under it and no
 * track chosen: Screen and Camera are their track chosen instead, which
 * takes over from it. */
export type RecordingPanelMode =
  "captions" | "cursor" | "keyboard" | "scene" | "sound" | "transcript";

/** What a recording's own picture is of: its screen, the camera where it
 * is of the camera alone, or none for audio. */
export type RecordingPicture = "camera" | "screen" | null;

/** The pen or Select the column shows pressed for `tool`: Select under the
 * crop, which is the chosen layer's rather than the column's. */
export const columnTool = (tool: RecordingCanvasTool): EditorTool => {
  const pen = drawingToolKind(tool);
  if (pen === "spotlight") return "focus";
  return pen ?? "select";
};

/** The mode the title bar shows open: the one the picture's tool belongs to,
 * else, under Select, the chosen track's (so choosing the screen or the
 * camera opens Screen or Camera, the other switches, and letting it go puts
 * it away), else the island-only mode open. A recording of the camera alone
 * has its camera as its own picture, the primary track, which opens
 * Camera. */
export const columnMode = ({
  panel,
  picture,
  tool,
  track,
}: {
  panel: RecordingPanelMode | null;
  /** What the recording's own picture is of. */
  picture: RecordingPicture;
  tool: RecordingCanvasTool;
  track: RecordingVideoTrackId | null;
}): EditorMode | null => {
  switch (tool) {
    case "canvas":
      return "frame";
    case "select":
      return track === "camera" || (track === "primary" && picture === "camera")
        ? "camera"
        : track === "primary"
          ? "screen"
          : panel;
    default:
      return null;
  }
};

/** What a mode takes up: the picture's tool, the track it chooses (none for
 * an island-only mode, Scene among them, which a chosen track would
 * override; left as it is for Frame), and the island-only mode; none goes
 * back to Select. Camera chooses the recording's own track where that is the
 * camera. */
export const modeTool = (
  mode: EditorMode | null,
  picture: RecordingPicture,
): {
  panel: RecordingPanelMode | null;
  tool: RecordingCanvasTool;
  track?: RecordingVideoTrackId | null;
} => {
  if (mode === null) return { panel: null, tool: "select" };
  switch (mode) {
    case "camera":
      return {
        panel: null,
        tool: "select",
        track: picture === "camera" ? "primary" : "camera",
      };
    case "frame":
      return { panel: null, tool: "canvas" };
    case "screen":
      return { panel: null, tool: "select", track: "primary" };
  }
  return { panel: mode, tool: "select", track: null };
};

/** Which island the tool in hand brings out, and the tool that brought
 * it, which tells one pen's island from another's: a pen's own, or the
 * annotations chosen with it or with Select's band; the crop's; else the open
 * mode's. Nothing in hand with Select brings out nothing. */
export const recordingIsland = ({
  annotationCount,
  mode,
  selectedKind,
  tool,
}: {
  annotationCount: number;
  mode: EditorMode | null;
  selectedKind: AnnotationKind | null;
  tool: RecordingCanvasTool;
}): { island: EditorIslandKind; tool: string } | null => {
  const pen = drawingToolKind(tool);
  if (pen !== null)
    return {
      island: "annotation",
      tool: pen === "spotlight" ? "focus" : pen,
    };
  if (annotationCount > 1) return { island: "annotation", tool: "select" };
  if (selectedKind !== null)
    return {
      island: "annotation",
      tool: selectedKind === "spotlight" ? "focus" : selectedKind,
    };
  if (tool === "crop") return { island: "crop", tool: "crop" };
  return mode === null ? null : { island: mode, tool: mode };
};
