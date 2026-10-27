// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimeMap } from "../model/timeline-model";

// Where a moment falls along the strip, as a fraction of its width: to
// scale, in the edited recording's time, as it plays. A second is the same
// width everywhere, so the playhead crosses the strip at an even pace and
// what is squeezed is drawn as short as it plays. The strip shows the part
// of the recording in view (`use-view.ts`); the overview under it shows the
// whole, at the same scale.

/** The part of the edited recording in view, as shares of its length. */
export type View = { end: number; start: number };

export type StripScale = {
  /** The recorded moment under a fraction of the strip's width. */
  toSeconds: (x: number) => number;
  /** Fraction of the strip's width for a recorded moment: 0 to 1 where it
   * is in view, past them where it is not. */
  toX: (seconds: number) => number;
};

export function stripScale({
  map,
  view,
}: {
  map: TimeMap;
  view: View;
}): StripScale {
  const length = Math.max(map.duration, 1e-6);
  const span = Math.max(view.end - view.start, 1e-6);
  return {
    toSeconds: (x) =>
      map.toOriginal(
        (view.start + Math.min(Math.max(x, 0), 1) * span) * length,
      ),
    toX: (seconds) => (map.toTime(seconds) / length - view.start) / span,
  };
}

/** The least width, in px, of anything on the strip that can be pressed. */
const MIN_HIT_PX = 24;

/** A span's place in px, reaching past its drawn width when that is too
 * narrow to press; `pad` is how far it reaches on each side. */
export const reach = (
  scale: StripScale,
  { end, start }: { end: number; start: number },
  width: number,
) => {
  const x = scale.toX(start) * width;
  const px = Math.max(0, (scale.toX(end) - scale.toX(start)) * width);
  const pad = Math.max(0, (MIN_HIT_PX - px) / 2);
  return {
    pad,
    px,
    style: { insetInlineStart: x - pad, width: px + pad * 2 },
  };
};

/** A span's place along the strip, as a style. */
export const placed = (scale: StripScale, start: number, end: number) => ({
  insetInlineStart: `${String(scale.toX(start) * 100)}%`,
  width: `${String(Math.max(0, scale.toX(end) - scale.toX(start)) * 100)}%`,
});

/** A moment's place along the strip, as a style. */
export const at = (scale: StripScale, seconds: number) => ({
  insetInlineStart: `${String(scale.toX(seconds) * 100)}%`,
});
