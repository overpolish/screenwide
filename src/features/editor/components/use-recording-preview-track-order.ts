// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { RecordingVideoTrackId } from "../types";

/** Moving a video track through the layer order, from the canvas menu or from
 * the keyboard. */
export function useRecordingPreviewTrackOrder({
  activeVideoTrack,
  onVideoTrackOrderChange,
  videoTrackOrder,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  videoTrackOrder: readonly RecordingVideoTrackId[];
  onVideoTrackOrderChange?: (tracks: RecordingVideoTrackId[]) => void;
}) {
  const moveVideoTrack = useCallback(
    (track: RecordingVideoTrackId, direction: "backward" | "forward") => {
      const currentIndex = videoTrackOrder.indexOf(track);
      const nextIndex =
        direction === "forward" ? currentIndex - 1 : currentIndex + 1;
      if (
        currentIndex < 0 ||
        nextIndex < 0 ||
        nextIndex >= videoTrackOrder.length
      )
        return;
      const next = [...videoTrackOrder];
      [next[currentIndex], next[nextIndex]] = [
        next[nextIndex],
        next[currentIndex],
      ];
      onVideoTrackOrderChange?.(next);
    },
    [onVideoTrackOrderChange, videoTrackOrder],
  );
  const moveActiveVideoTrack = useCallback(
    (direction: "backward" | "forward") => {
      if (activeVideoTrack) moveVideoTrack(activeVideoTrack, direction);
    },
    [activeVideoTrack, moveVideoTrack],
  );
  // The shortcut hook re-binds its window listener whenever a handler identity
  // changes, so these stay stable across the per-move draft renders.
  const moveActiveVideoTrackBackward = useCallback(() => {
    moveActiveVideoTrack("backward");
  }, [moveActiveVideoTrack]);
  const moveActiveVideoTrackForward = useCallback(() => {
    moveActiveVideoTrack("forward");
  }, [moveActiveVideoTrack]);
  return {
    moveActiveVideoTrackBackward,
    moveActiveVideoTrackForward,
    moveVideoTrack,
  };
}
