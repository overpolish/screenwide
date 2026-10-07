// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { ownsTextEditingKeys } from "../../shortcuts/keyboard-target";
import { Playhead } from "../scrub-playhead";
import { SeekHandler } from "../timeline-seek";

import {
  adjacentRecordingMoment,
  VisibleRecordingMoment,
} from "./recording-moments";

/**
 * Option (Alt on Windows) with Left or Right takes the playhead to the
 * previous or next moment. The window's own arrow handling leaves arrows with
 * Option alone, and a text field keeps them for moving by word.
 */
export function useMomentNavigation({
  moments,
  onSeek,
  playhead,
}: {
  moments: readonly VisibleRecordingMoment[];
  onSeek: SeekHandler;
  playhead: Playhead;
}) {
  const latestRef = useRef({ moments, onSeek, playhead });
  latestRef.current = { moments, onSeek, playhead };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        !event.altKey ||
        event.metaKey ||
        event.ctrlKey ||
        event.shiftKey ||
        event.isComposing ||
        (event.code !== "ArrowLeft" && event.code !== "ArrowRight") ||
        ownsTextEditingKeys(event.target)
      )
        return;
      const { moments, onSeek, playhead } = latestRef.current;
      const target = adjacentRecordingMoment(
        moments,
        playhead.ratio(),
        event.code === "ArrowRight" ? 1 : -1,
      );
      event.preventDefault();
      event.stopPropagation();
      if (target === null) return;
      onSeek(target, "start");
      onSeek(target, "end");
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, []);
}
