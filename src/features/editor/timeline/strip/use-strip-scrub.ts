// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { type TimelineScale } from "../timeline-scale";
import { type ScrubPhase } from "../timeline-seek";

// Pressing the empty strip moves the playhead there and lets the selection
// go; held, the playhead follows the pointer, one scrub from the press to
// the release. The parts on the strip stop their own presses, so only the
// empty strip and the empty lanes reach it.

export function useStripScrub({
  onDeselect,
  onSeek,
  scale,
}: {
  onDeselect: () => void;
  /** Told the recorded moment to move the playhead to, and where in a
   * scrub it falls. */
  onSeek: (seconds: number, phase: ScrubPhase) => void;
  /** The track's scale, across the whole recording. */
  scale: TimelineScale;
}) {
  const trackRef = useRef<HTMLDivElement>(null);
  const isScrubbingRef = useRef(false);

  const fractionOf = (event: { clientX: number }) => {
    const box = trackRef.current?.getBoundingClientRect();
    if (!box || box.width === 0) return 0;
    return (event.clientX - box.left) / box.width;
  };
  const secondsAt = (event: React.PointerEvent) =>
    scale.toSeconds(fractionOf(event));
  const release = (event: React.PointerEvent) => {
    if (!isScrubbingRef.current) return;
    isScrubbingRef.current = false;
    onSeek(secondsAt(event), "end");
  };

  return {
    fractionOf,
    handlers: {
      onPointerCancel: release,
      onPointerDown: (event: React.PointerEvent<HTMLElement>) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        isScrubbingRef.current = true;
        onSeek(secondsAt(event), "start");
        onDeselect();
      },
      onPointerMove: (event: React.PointerEvent) => {
        if (isScrubbingRef.current) onSeek(secondsAt(event), "move");
      },
      onPointerUp: release,
    },
    /** Told each pointer event of a drag on the playhead's knob. */
    scrub: (event: React.PointerEvent, phase: ScrubPhase) => {
      onSeek(secondsAt(event), phase);
    },
    secondsAt,
    trackRef,
  };
}
