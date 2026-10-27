// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";

// Still time drawn as what it behaves like: a coil spring, seen from the
// side and a little from above, so each turn is a loop with its far half
// fainter than its near one. A pause has as many coils as its length
// earns, and keeps them however it is squeezed, so squeezing packs them
// tighter until they overlap, and room spreads them out, as a spring
// compresses and stretches. Drawn in pixels at the width it is given, so a
// coil is round at any width, not stretched with the box.

/** Coils a second of pause earns, and the least and most a spring has. */
const COILS_PER_S = 1.4;
const MIN_COILS = 2;
const MAX_COILS = 18;
/** The tightest a coil packs, in px between turns, past which it would
 * only draw a solid bar. */
const MIN_PITCH_PX = 2.4;
/** Samples along each turn: enough for a smooth curve at strip sizes. */
const SAMPLES = 18;
/** How far each turn leans toward the eye, as a share of its height:
 * enough to open the loops, not so much the spring looks flat. */
const LEAN = 0.32;
const HEIGHT = 16;
/** Straight wire at each end before the first turn, in px. */
const LEAD_PX = 2;

type Point = { x: number; y: number };

/** The wire, as runs of its near half and its far half, and where along
 * the width the coiled part ends. */
function coil(width: number, seconds: number) {
  const radius = HEIGHT / 2 - 1;
  const room = width - LEAD_PX * 2;
  // A turn leans forward by up to twice this, so the coiled part is that
  // much shorter and its loops stay inside the width.
  const lean = Math.min(radius * 2 * LEAN, room / 4);
  const span = room - lean * 2;
  const coils = Math.max(
    1,
    Math.min(
      Math.round(
        Math.min(Math.max(seconds * COILS_PER_S, MIN_COILS), MAX_COILS),
      ),
      Math.floor(span / MIN_PITCH_PX),
    ),
  );
  const runs: { isNear: boolean; points: Point[] }[] = [];
  const turns = coils * Math.PI * 2;
  const total = coils * SAMPLES;
  for (let index = 0; index <= total; index += 1) {
    const t = (index / total) * turns;
    // Along the axis, with the turn's lean swinging it forward and back a
    // quarter turn out of step with its rise and fall: that is what draws
    // a loop. Both start and end on the axis, meeting the straight wire.
    const point = {
      x: LEAD_PX + (t / turns) * span + (1 - Math.cos(t)) * lean,
      y: HEIGHT / 2 - Math.sin(t) * radius,
    };
    const isNear = Math.cos(t) >= 0;
    const last = runs[runs.length - 1] as (typeof runs)[number] | undefined;
    if (last?.isNear === isNear) last.points.push(point);
    else {
      // Each run starts where the last ended, so the wire is unbroken.
      const joint = last?.points[last.points.length - 1];
      runs.push({ isNear, points: joint ? [joint, point] : [point] });
    }
  }
  return { end: LEAD_PX + span, runs };
}

const path = (points: Point[]) =>
  points
    .map(
      ({ x, y }, index) =>
        `${index === 0 ? "M" : "L"}${x.toFixed(2)} ${y.toFixed(2)}`,
    )
    .join(" ");

export function Spring({
  look,
  seconds,
  width,
}: {
  /** A kept pause is drawn firm, as set; dead air a little stronger than
   * an ordinary pause, to be noticed. */
  look: "dead-air" | "kept" | "plain";
  /** How long the pause was recorded, which sets its coils. */
  seconds: number;
  /** Its drawn width, in px. */
  width: number;
}) {
  if (width < PACKED_BELOW_PX) return <PackedSpring look={look} />;
  const { end, runs } = coil(width, seconds);
  const middle = HEIGHT / 2;
  return (
    <svg
      aria-hidden
      className={cn(
        "absolute start-0 top-1/2 -translate-y-1/2 overflow-visible",
        INK[look],
      )}
      fill="none"
      height={HEIGHT}
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      viewBox={`0 0 ${String(width)} ${String(HEIGHT)}`}
      width={width}
    >
      <path
        d={`M0 ${String(middle)} L${String(LEAD_PX)} ${String(middle)} M${String(end)} ${String(middle)} L${String(width)} ${String(middle)}`}
        strokeWidth={1.25}
      />
      {runs.map(({ isNear, points }) => (
        <path
          d={path(points)}
          // The far half of each turn sits behind: fainter and finer.
          key={`${String(isNear)}-${points[0].x.toFixed(2)}`}
          opacity={isNear ? 1 : 0.45}
          strokeWidth={isNear ? 1.25 : 1}
        />
      ))}
    </svg>
  );
}

const INK: Record<Parameters<typeof Spring>[0]["look"], string> = {
  "dead-air": "text-fg-secondary",
  kept: "text-fg",
  plain: "text-fg-tertiary",
};

/** Below this width, in px, a pause has no room for its coils. */
const PACKED_BELOW_PX = 12;
/** How wide a packed spring is drawn, in px, whatever its room. */
const PACKED_PX = 9;

/** A spring with no room to stretch out, drawn as one squeezed shut: its
 * turns pressed together, seen side on. Drawn at its own small width over
 * the middle of its room, which may be narrower, so a pause squeezed to
 * nothing still shows there is one there to press. */
function PackedSpring({
  look,
}: {
  look: Parameters<typeof Spring>[0]["look"];
}) {
  const turns = [1.5, 4.5, 7.5];
  return (
    <svg
      aria-hidden
      className={cn(
        "absolute start-1/2 top-1/2 -translate-1/2 overflow-visible",
        INK[look],
      )}
      fill="none"
      height={HEIGHT}
      stroke="currentColor"
      strokeLinecap="round"
      viewBox={`0 0 ${String(PACKED_PX)} ${String(HEIGHT)}`}
      width={PACKED_PX}
    >
      {turns.map((x) => (
        <g key={x}>
          {/* The near half of each turn, then the far half behind it. */}
          <path d={`M${String(x)} 1 a1.5 7 0 0 1 0 14`} strokeWidth={1.25} />
          <path
            d={`M${String(x)} 15 a1.5 7 0 0 1 0 -14`}
            opacity={0.45}
            strokeWidth={1}
          />
        </g>
      ))}
    </svg>
  );
}
