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
  // Crop put away by letting the track go (Escape) still closes and eases
  // the turn back in on the track it was on.
  const [lastTrack, setLastTrack] = useState(activeVideoTrack);
  if (activeVideoTrack !== null && activeVideoTrack !== lastTrack)
    setLastTrack(activeVideoTrack);
  const turnedTrack = activeVideoTrack ?? lastTrack;
  const isScreen = turnedTrack === "primary";
  const { isFlat, open, share } = useCropTurn(
    canvasTool === "crop",
    isScreen && isTurned(arrangement ? arrangement.rotation : screenRotation),
  );
  const screenShare = isScreen ? share : 1;
  const isClosed = canvasTool !== "crop" && open === 0;
  useEffect(() => {
    if (isClosed) releaseSceneCrop();
  }, [isClosed]);
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
