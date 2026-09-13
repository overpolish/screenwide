// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef, useState } from "react";

/** Share transport intent between React controls and native event handlers. */
export function useRecordingPlaybackStatus() {
  const isPlayingRef = useRef(false);
  const [isPlaying, setIsPlaying] = useState(false);
  const updatePlaying = useCallback((playing: boolean) => {
    isPlayingRef.current = playing;
    setIsPlaying(playing);
  }, []);
  const confirmPlaying = useCallback(() => {
    isPlayingRef.current = true;
    setIsPlaying(true);
  }, []);
  return {
    confirmPlaying,
    isPlaying,
    isPlayingRef,
    setIsPlaying,
    updatePlaying,
  };
}
