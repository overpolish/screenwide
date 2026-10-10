// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, SetStateAction, useEffect, useRef, useState } from "react";

import { SeekHandler } from "../../timeline/timeline-seek";

import { usePreviewStill } from "./preview-still-store";

/**
 * Once playback stops, the preview's position settles on where the player
 * stopped. It is taken in the same render that sees playback stop, so the
 * selection and the Scene panel never draw a frame at the position playback
 * began from, and again a frame later for the player's final word. Where it
 * stands still is told to tools outside the preview, and where they ask it
 * to go, it goes.
 */
export function usePausedPreviewPosition(
  {
    getPositionMs,
    isPlaying,
    seek,
  }: { getPositionMs: () => number; isPlaying: boolean; seek: SeekHandler },
  positionMs: number,
  setPreviewPositionMs: Dispatch<SetStateAction<number>>,
) {
  const [wasPlaying, setWasPlaying] = useState(isPlaying);
  if (wasPlaying !== isPlaying) {
    setWasPlaying(isPlaying);
    if (!isPlaying) setPreviewPositionMs(getPositionMs());
  }
  useEffect(() => {
    if (isPlaying) return;
    const frame = requestAnimationFrame(() => {
      setPreviewPositionMs(getPositionMs());
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [getPositionMs, isPlaying, setPreviewPositionMs]);
  useEffect(() => {
    if (!isPlaying) usePreviewStill.getState().setPositionMs(positionMs);
  }, [isPlaying, positionMs]);
  // Gone to once for each request, however often the player's seek changes,
  // and let go of, so a preview opened later does not go there too.
  const seekRef = useRef(seek);
  seekRef.current = seek;
  const wanted = usePreviewStill((still) => still.wanted);
  useEffect(() => {
    if (!wanted) return;
    usePreviewStill.setState({ wanted: null });
    seekRef.current(wanted.positionMs, "start");
    seekRef.current(wanted.positionMs, "end");
  }, [wanted]);
}
