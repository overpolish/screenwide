// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { motionDurations } from "../../../../lib/motion";
import { type TimelineScale } from "../timeline-scale";

import { type StripBeat, type StripLull } from "./strip-beat-row";
import { type TakenOut } from "./taken-out";
import { usePointerHeld } from "./use-pointer-held";

// The strip remade from other parts at once, as when the screen's track
// leaves the export or comes back, or a cut, a restore or the squeeze
// changes how much of it plays, reshaped as one rearrangement rather than
// a jump: what stays glides to its new place, as a move's room does, what
// is new fades in where it lands, and what went fades out where it was,
// drawn as the strip last showed it. A change a drag makes is taken live.

type Shape = {
  beats: StripBeat[];
  lulls: StripLull[];
  scale: TimelineScale;
  /** The track's width, in px. */
  width: number;
};

/** A reshape of the strip, while it is under way. */
export type StripReshape = {
  isReshaping: boolean;
  /** What went, as it was drawn; null with no reshape under way. */
  leaving: Shape | null;
};

export function useStripReshape({
  beats,
  isScreenShown,
  lulls,
  scale,
  taken,
  width,
}: Shape & {
  /** Whether the screen's track is in the export. */
  isScreenShown: boolean;
  /** What is taken out of the recording. */
  taken: TakenOut;
}): StripReshape {
  // Names what the strip is made from and how much of it plays; a new key
  // reshapes it, unless a pointer held down is making the change.
  const key = [
    isScreenShown ? "screen" : "no-screen",
    taken.deleted.toFixed(3),
    taken.squeezed.toFixed(3),
  ].join(" ");
  const pointerHeld = usePointerHeld();
  // The strip as last drawn, which a reshape starts from.
  const lastRef = useRef<Shape>({ beats, lulls, scale, width });
  useLayoutEffect(() => {
    lastRef.current = { beats, lulls, scale, width };
  });
  // Seen while rendering, so the very frame that draws the new shape
  // already glides to it.
  const [reshape, setReshape] = useState<{
    from: Shape | null;
    key: string;
  }>({ from: null, key });
  if (reshape.key !== key)
    setReshape({ from: pointerHeld.current ? null : lastRef.current, key });
  const { from } = reshape;
  useEffect(() => {
    if (!from) return;
    const timer = setTimeout(() => {
      setReshape((current) => ({ ...current, from: null }));
    }, motionDurations.rearrange * 1000);
    return () => {
      clearTimeout(timer);
    };
  }, [from]);
  if (!from) return { isReshaping: false, leaving: null };
  const kept = new Set([...beats, ...lulls].map(({ id }) => id));
  return {
    isReshaping: true,
    leaving: {
      ...from,
      beats: from.beats.filter(({ id }) => !kept.has(id)),
      lulls: from.lulls.filter(({ id }) => !kept.has(id)),
    },
  };
}
