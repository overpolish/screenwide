// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The clip's island draws its sound through a lens, as the editor's fisheye
// exploration drew a timeline: precision without zoom controls. The lens
// rides with the clip's window: the stretch the clip plays always fills a
// good share of the width, wherever the window stands, and the sound before
// and after it is squeezed into what is left either side, so the whole
// sound stays in view. Carried, the window moves the lens with it.

/** The least share of the width the clip's stretch takes. */
const WINDOW_SHARE = 0.4;

/** How a sound is laid across a width with the clip's stretch magnified
 * where the window stands: where a moment is drawn, and which moment is
 * under a point. */
export type Lens = {
  toMs: (x: number) => number;
  toX: (ms: number) => number;
};

/** How wide the window is, in px, on a track `width` px wide: its share of
 * the sound, or the least share, whichever is more. */
export const windowPx = (width: number, clipMs: number, soundMs: number) =>
  width *
  Math.min(Math.max(WINDOW_SHARE, soundMs > 0 ? clipMs / soundMs : 1), 1);

/** Where the window stands with time spread evenly either side of it: as
 * far along the room it leaves as it is along the sound. */
export function evenStart({
  fromMs,
  latestMs,
  room,
}: {
  fromMs: number;
  /** The latest the window can start, in ms. */
  latestMs: number;
  /** The width the window leaves, in px. */
  room: number;
}) {
  return latestMs > 0 ? (fromMs / latestMs) * room : 0;
}

/** The window standing at `startX`, kept inside the track, and against its
 * start or end where no sound is left before or after the window. */
export function keptStart({
  atEnd,
  atStart,
  room,
  startX,
}: {
  atEnd: boolean;
  atStart: boolean;
  room: number;
  startX: number;
}) {
  if (atStart) return 0;
  if (atEnd) return room;
  return Math.min(Math.max(startX, 0), room);
}

/** How far the window's start moves in the sound for the pointer `px` px
 * from where it took hold: a pixel of the window's own magnified time near
 * there, for fine adjustment, rising so that the pointer reaching the
 * track's end brings it to the sound's, and the pointer never has to go
 * further than the track does. */
export function carried({
  finePerPx,
  px,
  remainingMs,
  roomPx,
}: {
  /** How many ms a pixel of the window holds. */
  finePerPx: number;
  px: number;
  /** How much sound is left that way, in ms. */
  remainingMs: number;
  /** How far the pointer can go that way on the track, in px. */
  roomPx: number;
}) {
  const distance = Math.abs(px);
  // At the track's end already, with sound still beyond, it moves finely.
  if (roomPx < 1 || remainingMs <= 0)
    return Math.sign(px) * Math.min(distance * finePerPx, remainingMs);
  const reach = Math.min(distance / roomPx, 1);
  const fine = Math.min((finePerPx * roomPx) / remainingMs, 1);
  return Math.sign(px) * remainingMs * (fine * reach + (1 - fine) * reach ** 3);
}

/** How far either side of the window the zoom takes to fall away, against
 * the clip's own length. */
const FALL_PER_CLIP = 1;
/** The most of a side's width the falling zoom may take. */
const FALL_SHARE = 0.6;
const HALF_ROOT_PI = Math.sqrt(Math.PI) / 2;

/** The error function, to Abramowitz and Stegun's 7.1.26, close enough to
 * lay pixels by. */
function erf(x: number) {
  const t = 1 / (1 + 0.3275911 * Math.abs(x));
  const y =
    1 -
    ((((1.061405429 * t - 1.453152027) * t + 1.421413741) * t - 0.284496736) *
      t +
      0.254829592) *
      t *
      Math.exp(-x * x);
  return Math.sign(x) * y;
}

/** Where a moment `ms` past the window's edge falls, in px past it, on a
 * side `sideMs` long laid across `sidePx`: the window's own zoom at the
 * edge, falling smoothly away to an even spread across the rest. */
function sideLayout({
  clipMs,
  insidePerMs,
  sideMs,
  sidePx,
}: {
  clipMs: number;
  /** The window's zoom, in px a ms. */
  insidePerMs: number;
  sideMs: number;
  sidePx: number;
}) {
  if (sideMs <= 0 || sidePx <= 0 || insidePerMs <= 0) return () => 0;
  const fall = Math.min(
    clipMs * FALL_PER_CLIP,
    (FALL_SHARE * sidePx) / (insidePerMs * HALF_ROOT_PI),
    sideMs,
  );
  // How much the falling zoom adds past the edge, and the even spread the
  // rest of the side takes so the whole side fills its width.
  const swell = (ms: number) => fall * HALF_ROOT_PI * erf(ms / fall);
  const spread =
    (sidePx - insidePerMs * swell(sideMs)) /
    Math.max(sideMs - swell(sideMs), 1);
  return (ms: number) => spread * ms + (insidePerMs - spread) * swell(ms);
}

export function fisheye({
  fromMs,
  soundMs,
  startX,
  toMs,
  width,
  window,
}: {
  /** Where the clip's stretch starts and ends, in ms. */
  fromMs: number;
  soundMs: number;
  /** Where the window stands, in px. */
  startX: number;
  toMs: number;
  width: number;
  /** The window's width, in px. */
  window: number;
}): Lens {
  const endX = startX + window;
  const clipMs = toMs - fromMs;
  const insidePerMs = clipMs > 0 ? window / clipMs : 0;
  const before = sideLayout({
    clipMs,
    insidePerMs,
    sideMs: fromMs,
    sidePx: startX,
  });
  const after = sideLayout({
    clipMs,
    insidePerMs,
    sideMs: soundMs - toMs,
    sidePx: width - endX,
  });
  const toX = (ms: number) => {
    if (ms < fromMs) return startX - before(fromMs - ms);
    if (ms <= toMs) return startX + (ms - fromMs) * insidePerMs;
    return endX + after(ms - toMs);
  };
  return {
    toMs: (x) => {
      // Found by halving, as the falling zoom has no inverse to write out.
      let low = 0;
      let high = soundMs;
      for (let step = 0; step < 40; step += 1) {
        const middle = (low + high) / 2;
        if (toX(middle) < x) low = middle;
        else high = middle;
      }
      return (low + high) / 2;
    },
    toX,
  };
}

/** The share of the quietest levels left out when the levels are stretched
 * for drawing, so a silent intro does not flatten the rest. */
const QUIET_SHARE = 0.05;

/** `levels` stretched between the sound's quieter passages and its
 * loudest, so they read apart however loud the sound is mastered. */
export function contrasted(levels: number[]) {
  if (levels.length === 0) return levels;
  const sorted = [...levels].sort((one, other) => one - other);
  const quiet = sorted[Math.floor(QUIET_SHARE * (sorted.length - 1))];
  const loudest = sorted[sorted.length - 1];
  if (loudest - quiet < 1e-3) return levels.map(() => 0.5);
  return levels.map((level) =>
    Math.min(Math.max((level - quiet) / (loudest - quiet), 0.04), 1),
  );
}

/** How far one press of an arrow key moves the window, in ms, and with ⌘
 * or Ctrl held. */
const KEY_STEP_MS = { coarse: 1_000, fine: 100 };

/** Where `key` moves the window's start from `offsetMs`: a second at a
 * time with the arrow keys, a tenth with ⌘ or Ctrl held, or to either end
 * with Home and End; null for any other key. */
export function keyedStart({
  isFine,
  key,
  latestMs,
  offsetMs,
}: {
  isFine: boolean;
  key: string;
  latestMs: number;
  offsetMs: number;
}) {
  if (key === "Home") return 0;
  if (key === "End") return latestMs;
  const sign =
    key === "ArrowLeft" || key === "ArrowDown"
      ? -1
      : key === "ArrowRight" || key === "ArrowUp"
        ? 1
        : 0;
  if (sign === 0) return null;
  const step = isFine ? KEY_STEP_MS.fine : KEY_STEP_MS.coarse;
  return Math.round(Math.min(Math.max(offsetMs + sign * step, 0), latestMs));
}
