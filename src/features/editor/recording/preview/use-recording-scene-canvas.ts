// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { Arrangement } from "../../annotations/annotation-order";
import { RecordingVideoTrackId } from "../../types";
import { RecordingScenes } from "../scenes/use-recording-scenes";

/**
 * What the preview does with scenes beyond drawing them. Whatever else is
 * chosen, from the lanes or the picture, lets the chosen scenes go, so Delete
 * never takes away two kinds of thing at once; choosing scenes clears the
 * rest from the scene lane itself. The custom scene under the playhead
 * reorders its panes from the canvas, the selected pane by the Arrange keys
 * and either pane by its menu, while it plays: a paused scene draws nothing
 * to put in front.
 */
export function useRecordingSceneCanvas({
  activeVideoTrack,
  otherSelectionMade,
  scenes,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  otherSelectionMade: boolean;
  scenes: RecordingScenes;
}) {
  const clearScenes = scenes.selection.onClear;
  useEffect(() => {
    if (otherSelectionMade) clearScenes();
  }, [clearScenes, otherSelectionMade]);
  const orderedScene = scenes.canPlaceCamera ? scenes.current : null;
  const pane =
    activeVideoTrack === "camera"
      ? "camera"
      : activeVideoTrack === "primary"
        ? "screen"
        : null;
  return {
    arrangeSelectedPane:
      pane && orderedScene?.boxes?.camera
        ? (move: Arrangement) => {
            scenes.controls.arrangePane(pane, move);
          }
        : undefined,
    paneMenu: { arrange: scenes.controls.arrangePane, clip: orderedScene },
  };
}
