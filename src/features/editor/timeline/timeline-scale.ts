// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Where a moment falls along the strip, as a fraction of its width. Every
// part is given times as recorded, so a cut or a squeeze never moves what
// they belong to; the scale draws them in the edited recording's time, as
// it plays, so a second is the same width everywhere and what is squeezed
// is drawn as short as it plays.

/** A stretch of the recording, in recorded seconds. */
export type Span = { end: number; start: number };

/** The part of the edited recording in view, as shares of its length. */
export type View = { end: number; start: number };

/** How the recording's time maps to the time it plays in: `duration` is
 * how long the edited recording plays. Reordered, a recorded moment can be
 * where one stretch ends and, played elsewhere, where another starts; asked
 * for an `isEnd`, the map places the moment as the end of what stops there.
 */
export type TimeMap = {
  duration: number;
  toPlayed: (recorded: number, isEnd?: boolean) => number;
  toRecorded: (played: number) => number;
};

export type TimelineScale = {
  /** The recorded moment under a fraction of the strip's width. */
  toSeconds: (x: number) => number;
  /** Fraction of the strip's width for a recorded moment: 0 to 1 where it
   * is in view, past them where it is not; `isEnd` places it as the end of
   * what stops there. */
  toX: (seconds: number, isEnd?: boolean) => number;
};

/** The whole recording, as a view. */
export const WHOLE: View = { end: 1, start: 0 };

/** A recording that plays as it was recorded, `duration` seconds long. */
export const recordedTime = (duration: number): TimeMap => ({
  duration,
  toPlayed: (recorded) => recorded,
  toRecorded: (played) => played,
});

export function timelineScale(map: TimeMap, view: View = WHOLE): TimelineScale {
  const length = Math.max(map.duration, 1e-6);
  const span = Math.max(view.end - view.start, 1e-6);
  return {
    toSeconds: (x) =>
      map.toRecorded(
        (view.start + Math.min(Math.max(x, 0), 1) * span) * length,
      ),
    toX: (seconds, isEnd) =>
      (map.toPlayed(seconds, isEnd) / length - view.start) / span,
  };
}

/** `view` kept whole and inside the recording, no narrower than `least`. */
export function clampView(view: View, least: number): View {
  const span = Math.min(Math.max(view.end - view.start, least), 1);
  const start = Math.min(Math.max(view.start, 0), 1 - span);
  return { end: start + span, start };
}

/** The least width, in px, of anything on the strip that can be pressed. */
const MIN_HIT_PX = 24;

/** A span's place in px on a strip `width` px wide, reaching past its drawn
 * width when that is too narrow to press; `pad` is how far it reaches on
 * each side, and `px` its drawn width. */
export const reach = (scale: TimelineScale, span: Span, width: number) => {
  const x = scale.toX(span.start) * width;
  const px = Math.max(
    0,
    (scale.toX(span.end, true) - scale.toX(span.start)) * width,
  );
  const pad = Math.max(0, (MIN_HIT_PX - px) / 2);
  return {
    pad,
    px,
    style: { insetInlineStart: x - pad, width: px + pad * 2 },
  };
};

const percent = (share: number) => `${String(share * 100)}%`;

/** A span's place along the strip, as a style. */
export const placed = (scale: TimelineScale, start: number, end: number) => ({
  insetInlineStart: percent(scale.toX(start)),
  width: percent(Math.max(0, scale.toX(end, true) - scale.toX(start))),
});

/** A moment's place along the strip, as a style. */
export const at = (scale: TimelineScale, seconds: number) => ({
  insetInlineStart: percent(scale.toX(seconds)),
});
