// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTurned, SceneRotation, shareOfTurn } from "../../layer-rotation";
import { useCropTurn } from "../../preview/use-crop-turn";
import { RecordingSceneArrangement } from "../scenes/recording-scene-arrangement";

import type { RecordingCanvasTool } from "./recording-crop-toggle";
import type { CropTurn } from "../../preview/use-crop-turn";
import type { RecordingVideoTrackId } from "../../types";

/**
 * The crop tool's turn ease as the recording's preview takes it. Only the
 * screen turns, by the scene that places it or else by its own turn, so only
 * cropping the screen eases. Until the crop is shown the screen keeps the
 * select tool's chrome, which turns with it.
 */
export function useRecordingCropTurn<Tool extends RecordingCanvasTool>({
  activeVideoTrack,
  arrangement,
  canvasTool,
  screenRotation,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  arrangement: RecordingSceneArrangement | null | undefined;
  canvasTool: Tool;
  /** How the screen turns itself, outside every scene. */
  screenRotation: SceneRotation | undefined;
}) {
  const isScreen = activeVideoTrack === "primary";
  const { isFlat, share } = useCropTurn(
    canvasTool === "crop",
    isScreen && isTurned(arrangement ? arrangement.rotation : screenRotation),
  );
  const screenShare = isScreen ? share : 1;
  return {
    /** The scene under the playhead, its turn of the screen eased. */
    arrangement:
      arrangement && screenShare !== 1
        ? {
            ...arrangement,
            rotation: shareOfTurn(arrangement.rotation, screenShare),
          }
        : arrangement,
    /** The tool the selection chrome is drawn for. */
    chromeTool:
      canvasTool === "crop" && !isFlat ? ("select" as const) : canvasTool,
    /** The ease as the screen takes it: a camera in hand never turns. */
    cropTurn: { isFlat, share: screenShare } satisfies CropTurn,
    /** The screen's own turn, eased. */
    screenRotation:
      screenRotation && screenShare !== 1
        ? shareOfTurn(screenRotation, screenShare)
        : screenRotation,
  };
}
