// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, SetStateAction, useEffect, useState } from "react";

/**
 * Once playback stops, the preview's position settles on where the player
 * stopped. It is taken in the same render that sees playback stop, so the
 * selection and the Scene panel never draw a frame at the position playback
 * began from, and again a frame later for the player's final word.
 */
export function usePausedPreviewPosition({
  getPositionMs,
  isPlaying,
  setPreviewPositionMs,
}: {
  getPositionMs: () => number;
  isPlaying: boolean;
  setPreviewPositionMs: Dispatch<SetStateAction<number>>;
}) {
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
}
