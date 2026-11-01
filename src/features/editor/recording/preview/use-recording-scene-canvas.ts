// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

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

/** The scenes chosen belong to the Scene mode they take up: putting the mode
 * away lets them go, and once the scene chosen leaves the recording, deleted
 * by its key or its island, the mode is put away by `putAway`, with nothing
 * left for it to stand over. */
export function useSceneModeChoice({
  isSceneInHand,
  putAway,
  scenes,
}: {
  isSceneInHand: boolean;
  putAway: () => void;
  scenes: Pick<RecordingScenes, "clips" | "selection">;
}) {
  const chosenRef = useRef<ReadonlySet<string>>(new Set());
  const chosen = scenes.selection.ids;
  const { clips } = scenes;
  useEffect(() => {
    const wasChosen = chosenRef.current;
    chosenRef.current = chosen;
    const isGone = [...wasChosen].some(
      (id) => !clips.some((clip) => clip.id === id),
    );
    if (isGone && isSceneInHand) putAway();
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- only a change to the scenes or the choice can take the chosen one away; `putAway` is remade each render.
  }, [chosen, clips]);
  const letGo = scenes.selection.onClear;
  useEffect(() => {
    if (!isSceneInHand) letGo();
  }, [isSceneInHand, letGo]);
}
