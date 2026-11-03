// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GLIDE_MOVE_SECONDS, glideMoveEased } from "../../glide/glide-move";
import { type GlideRegion } from "../../glide/glide-regions";
import { cell } from "../../glide/island/glide-island-script";
import { type GlideIslandPhase } from "../../glide/island/glide-island-types";
import { type Box, lerp, progress } from "../preview/scenes/scene-kit";

import type { PictureSize } from "../preview/settings-preview-api";

// Glide's gestures in Settings' picture, each a timeline read from one
// clock. Moving a window to the side: the pointer takes it by its top bar,
// Glide's island comes out where the gesture began and opens onto the left
// half as the hand heads that way, then closes round its check as the
// window is sent there. Double-tapping to centre: the pointer comes to the
// window and double-clicks, and the window is centred on the screen. A
// window moves as Glide's own tween carries one, and the pointer comes with
// it if it follows.

export type GlideGesture = "center" | "side";

/** The screen's area windows are placed in, under the menu bar. */
export type WorkArea = PictureSize & { top: number };

export type GlideFrame = {
  /** Where the gesture began, which the island comes out over. */
  grip: { x: number; y: number };
  island: {
    phase: GlideIslandPhase;
    pulse: number;
    region: GlideRegion | null;
  } | null;
  /** The window the gesture moves, and the other one. */
  moving: Box;
  /** Where the pointer is, and whether it is pressed or hidden. */
  pointer: { hidden: boolean; pressed: boolean; x: number; y: number };
  still: Box;
};

/** One pass of each gesture, in seconds. */
export const GESTURE_LOOP: Record<GlideGesture, number> = {
  center: 3.4,
  side: 4.6,
};
/** Where the pointer takes a window's top bar, in points from its corner. */
const GRIP = { x: 90, y: 12 };
/** Side: the gesture starting, the pointer pinned and hidden; the hand let
 * go; and the pointer back, a beat into the island's fade, as the app
 * returns it 90 ms in. */
const PINNED = 0.8;
const SENT = 2.2;
const GONE = 2.7;
const RETURNED = GONE + 0.09;
/** Centre: the two clicks, and the window starting to move after them. */
const CLICKS = [0.9, 1.05];
const CLICK = 0.07;
const CENTRED = 1.15;

const leftHalf = cell({ gridCols: 2 });

/** A box placed by shares of the work area: start, top, width and height. */
const share = (
  work: WorkArea,
  [x, y, width, height]: readonly [number, number, number, number],
): Box => ({
  height: height * work.height,
  width: width * work.width,
  x: x * work.width,
  y: work.top + y * work.height,
});

const between = (from: Box, to: Box, amount: number): Box => ({
  height: lerp(from.height, to.height, amount),
  width: lerp(from.width, to.width, amount),
  x: lerp(from.x, to.x, amount),
  y: lerp(from.y, to.y, amount),
});

/** How far a move begun at `start` has gone at `at`. */
const moved = (at: number, start: number) =>
  glideMoveEased((at - start) / GLIDE_MOVE_SECONDS);

/** The pointer coming up from below to take `grip`. */
function reaching(at: number, work: WorkArea, grip: { x: number; y: number }) {
  const reach = progress(at, 0.1, 0.7);
  return {
    x: lerp(work.width * 0.5, grip.x, reach),
    y: lerp(work.top + work.height * 0.9, grip.y, reach),
  };
}

function islandAt(at: number): GlideFrame["island"] {
  if (at < PINNED || at >= GONE) return null;
  if (at >= SENT) return { phase: "committed", pulse: 1, region: null };
  return {
    phase: "gliding",
    // The hand rests on the left half a moment before it is sent.
    pulse: at >= 1.8 ? 1 : 0,
    region: at >= 1.2 ? leftHalf : null,
  };
}

/** Where everything is `at` seconds into a pass of `gesture`, or still
 * where it starts with Glide off. */
export function glideFrame({
  at,
  cursorFollows,
  gap,
  gesture,
  isOn,
  work,
}: {
  at: number;
  cursorFollows: boolean;
  /** The space Glide keeps round windows, in points. */
  gap: number;
  gesture: GlideGesture;
  isOn: boolean;
  work: WorkArea;
}): GlideFrame {
  const still = share(work, [0.56, 0.12, 0.38, 0.76]);
  const from = share(
    work,
    gesture === "side" ? [0.16, 0.1, 0.4, 0.78] : [0.04, 0.2, 0.4, 0.72],
  );
  const grip = { x: from.x + GRIP.x, y: from.y + GRIP.y };
  const rest = { hidden: false, pressed: false, ...grip };
  if (!isOn) return { grip, island: null, moving: from, pointer: rest, still };

  if (gesture === "center") {
    const centred = {
      ...from,
      x: (work.width - from.width) / 2,
      y: work.top + (work.height - from.height) / 2,
    };
    const moving = between(from, centred, moved(at, CENTRED));
    const pressed = CLICKS.some((click) => at >= click && at < click + CLICK);
    const pointer =
      at < 0.7
        ? reaching(at, work, grip)
        : cursorFollows
          ? { x: moving.x + GRIP.x, y: moving.y + GRIP.y }
          : grip;
    return {
      grip,
      island: null,
      moving,
      pointer: { hidden: false, pressed, ...pointer },
      still,
    };
  }

  // Sent to the left half, and the other window to the right, the space
  // set kept round both.
  const half = {
    height: work.height - gap * 2,
    width: work.width / 2 - gap * 1.5,
    y: work.top + gap,
  };
  const moving = between(from, { ...half, x: gap }, moved(at, SENT));
  const pointer =
    at < 0.7
      ? reaching(at, work, grip)
      : cursorFollows && at >= SENT
        ? { x: moving.x + GRIP.x, y: moving.y + GRIP.y }
        : grip;
  return {
    grip,
    island: islandAt(at),
    moving,
    pointer: {
      hidden: at >= PINNED && at < RETURNED,
      pressed: false,
      ...pointer,
    },
    still: { ...half, x: work.width / 2 + gap / 2 },
  };
}
