// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { springCoil } from "../../glyphs/spring-coil";

// A lull drawn as what it behaves like: a coil spring, seen from the side
// and a little from above. It has as many coils as its recorded length
// earns, and keeps them however it is squeezed, so squeezing packs them
// tighter and room spreads them out, as a spring compresses and stretches.
// Drawn in pixels at the width it is given, so a coil is round at any
// width, not stretched with the box.

/** Coils a second of lull earns, and the least and most a spring has. */
const COILS_PER_S = 1.4;
const MIN_COILS = 2;
const MAX_COILS = 18;
/** The tightest a coil packs, in px between turns, past which it would
 * only draw a solid bar. */
const MIN_PITCH_PX = 2.4;
/** How far each turn leans toward the eye, as a share of its height, which
 * `springCoil` holds the coiled part short by on each side. */
const LEAN = 0.32;
const HEIGHT = 16;
const RADIUS = HEIGHT / 2 - 1;
/** Straight wire at each end before the first turn, in px. */
const LEAD_PX = 2;

/** Below this width, in px, a lull has no room for its coils. */
const PACKED_BELOW_PX = 12;
/** How wide a packed spring is drawn, in px, whatever its room. */
const PACKED_PX = 9;

/** A kept lull is drawn firm, as set; dead air a little stronger than an
 * ordinary lull, to be noticed. */
export type SpringLook = "dead-air" | "kept" | "plain";

const INK: Record<SpringLook, string> = {
  "dead-air": "text-fg-secondary",
  kept: "text-fg",
  plain: "text-fg-tertiary",
};

export function SpringWire({
  gaps = [],
  look,
  seconds,
  width,
}: {
  look: SpringLook;
  /** How long the lull was recorded, which sets its coils. */
  seconds: number;
  /** Its drawn width, in px. */
  width: number;
  /** Where glyphs sit on the coil, in px along it: the wire fades out round
   * each, so the glyph reads clear of it. */
  gaps?: number[];
}) {
  if (width < PACKED_BELOW_PX) return <PackedSpring look={look} />;
  const room = width - LEAD_PX * 2;
  const span = room - Math.min(RADIUS * 2 * LEAN, room / 4) * 2;
  const coils = Math.max(
    1,
    Math.min(
      Math.round(
        Math.min(Math.max(seconds * COILS_PER_S, MIN_COILS), MAX_COILS),
      ),
      Math.floor(span / MIN_PITCH_PX),
    ),
  );
  const middle = HEIGHT / 2;
  const { end, runs } = springCoil({
    coils,
    lead: LEAD_PX,
    lean: LEAN,
    middle,
    radius: RADIUS,
    width,
  });
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
      style={gaps.length > 0 ? { maskImage: gapMask(gaps) } : undefined}
      viewBox={`0 0 ${String(width)} ${String(HEIGHT)}`}
      width={width}
    >
      <path
        d={`M0 ${String(middle)}H${String(LEAD_PX)}M${end.toFixed(2)} ${String(middle)}H${String(width)}`}
        strokeWidth={1.25}
      />
      {runs.map(({ d, isNear }) => (
        <path
          d={d}
          // The far half of each turn sits behind: fainter and finer.
          key={d}
          opacity={isNear ? 1 : 0.45}
          strokeWidth={isNear ? 1.25 : 1}
        />
      ))}
    </svg>
  );
}

/** Half the clear room, in px, the wire leaves round a glyph on it: the
 * glyph's width with a little air. */
const GAP_HALF_PX = 7;
/** How far, in px, the wire fades out on each side of that room, as a
 * scroll area fades its content at an edge rather than cutting it. */
const GAP_FADE_PX = 8;

/** A mask that fades the wire out round each of `gaps`, in px along it. */
function gapMask(gaps: number[]) {
  const px = (at: number) => `${String(at)}px`;
  const stops = [...gaps]
    .sort((first, second) => first - second)
    .flatMap((at) => [
      `#000 ${px(at - GAP_HALF_PX - GAP_FADE_PX)}`,
      `transparent ${px(at - GAP_HALF_PX)}`,
      `transparent ${px(at + GAP_HALF_PX)}`,
      `#000 ${px(at + GAP_HALF_PX + GAP_FADE_PX)}`,
    ]);
  return `linear-gradient(to right, #000, ${stops.join(", ")}, #000)`;
}

/** A spring with no room to stretch out, drawn as one squeezed shut: its
 * turns pressed together, seen side on. Drawn at its own small width over
 * the middle of its room, which may be narrower, so a lull squeezed to
 * nothing still shows there is one there to press. */
function PackedSpring({ look }: { look: SpringLook }) {
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
