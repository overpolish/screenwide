// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { type TimeMap } from "./timeline-model";

/** Plays by advancing the edited time, so a squeezed pause passes quickly
 * and a cut is skipped, as the export will play it; at the end it stops. */
export function usePlayback({
  isPlaying,
  map,
  onEnd,
  onPlayhead,
  playhead,
}: {
  isPlaying: boolean;
  map: TimeMap;
  onEnd: () => void;
  onPlayhead: (seconds: number) => void;
  playhead: number;
}) {
  const playheadRef = useRef(playhead);
  const handlersRef = useRef({ onEnd, onPlayhead });
  useEffect(() => {
    playheadRef.current = playhead;
    handlersRef.current = { onEnd, onPlayhead };
  });
  useEffect(() => {
    if (!isPlaying) return;
    let last = performance.now();
    let frame = 0;
    const tick = (now: number) => {
      const time = map.toTime(playheadRef.current) + (now - last) / 1000;
      last = now;
      if (time >= map.duration - 0.01) {
        handlersRef.current.onPlayhead(map.toOriginal(map.duration - 0.01));
        handlersRef.current.onEnd();
        return;
      }
      playheadRef.current = map.toOriginal(time);
      handlersRef.current.onPlayhead(playheadRef.current);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isPlaying, map]);
}
