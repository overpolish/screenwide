// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { isTurned, SceneRotation, shareOfTurn } from "../../layer-rotation";
import { useCropTurn } from "../../preview/use-crop-turn";
import { RecordingSceneArrangement } from "../scenes/recording-scene-arrangement";
import { releaseSceneCrop } from "../scenes/recording-scene-crop-hold";

import type { RecordingCanvasTool } from "./recording-canvas-tool";
import type { CropTurn } from "../../preview/use-crop-turn";
import type { RecordingVideoTrackId } from "../../types";

/**
 * The crop tool's ease as the recording's preview takes it: the track in
 * hand opens out around its crop and back, and only the screen turns, by the
 * scene that places it or else by its own turn, so only cropping the screen
 * eases a turn. The crop's chrome fades in and out with the opening. In a
 * scene the native preview opens the pane (`openPane`), and a preset held
 * where its panes stood is let go once the pane has closed onto where the
 * scene lays it.
 */
export function useRecordingCropTurn<Tool extends RecordingCanvasTool>({
  activeVideoTrack,
  arrangement,
  bakesCamera,
  canvasTool,
  isFramed,
  screenRotation,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  arrangement: RecordingSceneArrangement | null | undefined;
  /** Whether the camera is drawn into the picture, where a scene places it. */
  bakesCamera: boolean;
  canvasTool: Tool;
  /** Whether a scene places the panes at the playhead. */
  isFramed: boolean;
  /** How the screen turns itself, outside every scene. */
  screenRotation: SceneRotation | undefined;
}) {
  // The track the crop is open on, kept while it closes: put away by
  // letting the track go (Escape), it still closes and eases the turn back
  // in on that track, and a press on another layer closes this one's crop
  // before that one's opens, as putting the tool down and taking it up
  // would, rather than snapping one shut and the other open (user report).
  const [turnedTrack, setTurnedTrack] = useState(activeVideoTrack);
  const wantedTrack = activeVideoTrack ?? turnedTrack;
  const isSwitching = wantedTrack !== turnedTrack;
  const isScreen = turnedTrack === "primary";
  const { isFlat, open, share } = useCropTurn(
    canvasTool === "crop" && !isSwitching,
    isScreen && isTurned(arrangement ? arrangement.rotation : screenRotation),
  );
  if (isSwitching && open === 0) setTurnedTrack(wantedTrack);
  const screenShare = isScreen ? share : 1;
  const isClosed = canvasTool !== "crop" && open === 0;
  useEffect(() => {
    if (isClosed) releaseSceneCrop();
  }, [isClosed]);
  // A preset held while one layer is cropped is laid out once that crop has
  // closed for another layer's, so the next crop opens on the scene laid out
  // anew: held on, the first layer stayed where its crop window had moved it
  // while the second was cropped (user report).
  useEffect(() => releaseSceneCrop, [turnedTrack]);
  const openPane =
    isFramed && open > 0
      ? isScreen
        ? ("screen" as const)
        : turnedTrack === "camera" && bakesCamera
          ? ("camera" as const)
          : null
      : null;
  return {
    /** The scene under the playhead, its turn of the screen eased. */
    arrangement:
      arrangement && screenShare !== 1
        ? {
            ...arrangement,
            rotation: shareOfTurn(arrangement.rotation, screenShare),
          }
        : arrangement,
    /** The tool the selection chrome is drawn for: the crop's from the
     * moment the crop tool is taken up until the crop has closed, so its
     * chrome takes over from the select tool's on the same box as the
     * picture opens round it (`openingCropChrome`). */
    chromeTool:
      canvasTool === "crop" || open > 0 ? ("crop" as const) : canvasTool,
    /** The track the selection chrome is drawn on: the crop's, kept until
     * it has closed, so its chrome stays on the layer it is open on. */
    chromeTrack:
      canvasTool === "crop" || open > 0
        ? (turnedTrack ?? activeVideoTrack)
        : activeVideoTrack,
    /** The ease as the track takes it: a camera in hand never turns. */
    cropTurn: { isFlat, open, share: screenShare } satisfies CropTurn,
    /** The pane of the scene under the playhead the crop opens out, kept
     * while it closes, which the native preview draws. */
    openPane,
    /** The screen's own turn, eased. */
    screenRotation:
      screenRotation && screenShare !== 1
        ? shareOfTurn(screenRotation, screenShare)
        : screenRotation,
    /** The track whose turn eases, which outlives its choice while the
     * turn eases back in. */
    turnedTrack,
  };
}
