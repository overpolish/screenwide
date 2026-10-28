// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ISLAND } from "./island-geometry";

// The island's black as one outline: the row, and the section hanging under
// it, joined by curved inner corners. Drawn as one path, the shape has no
// seams to open between its parts, and every curve is exact. Coordinates run
// from the row's start edge, so a right-to-left layout mirrors the drawing.

export type IslandOutlineShape = {
  row: {
    /** Under a notch, how far the top flares out into the screen's top
     * edge on each side; none elsewhere. */
    flare: number;
    height: number;
    /** The bottom corners' radius. */
    radius: number;
    /** The top corners' radius; none where the row runs into the screen's
     * top edge under a notch. */
    top: number;
    width: number;
  };
  /** The section, by where it hangs: its start edge from the row's start,
   * its width, and how far it shows below the row. */
  section: { height: number; start: number; width: number } | null;
};

/** Below this much showing, a side that meets the row's end carries the
 * row's corner; above it, the section's. In between it blends. */
const CURVES = ISLAND.fillet + ISLAND.sectionRadius;

const n = (value: number) => String(Math.round(value * 100) / 100);
/** A rounded corner to `x`, `y`, turning the way the outline is walked. */
const round = (radius: number, x: number, y: number) =>
  `A${n(radius)} ${n(radius)} 0 0 1 ${n(x)} ${n(y)}`;
/** A join to `x`, `y`, curving the other way, into the shape. */
const hollow = (radius: number, x: number, y: number) =>
  `A${n(radius)} ${n(radius)} 0 0 0 ${n(x)} ${n(y)}`;

/** One side of the section, from where it leaves the row's bottom down to
 * where its bottom edge begins. `gap` is the row left beside the section on
 * that side. Where there is none, the side runs straight down and the row's
 * corner is carried down to the section's, growing from the row's radius to
 * the section's as the section opens. Elsewhere the row's corner and the
 * join take what room there is, so the outline changes continuously as the
 * section moves towards or away from the row's end. */
function sideRadii(gap: number, shown: number, row: IslandOutlineShape["row"]) {
  if (gap < 0.5) {
    const opened = Math.min(1, shown / CURVES);
    const carried = row.radius + (ISLAND.sectionRadius - row.radius) * opened;
    return {
      corner: Math.min(carried, row.height + shown - row.top),
      isMeeting: true,
      join: 0,
      rowCorner: 0,
    };
  }
  const rowCorner = Math.min(row.radius, gap);
  return {
    corner: ISLAND.sectionRadius,
    isMeeting: false,
    join: Math.min(ISLAND.fillet, Math.max(0, gap - rowCorner)),
    rowCorner,
  };
}

/** Where the join on a side that does not meet the row's end leaves the
 * row's bottom, and where it meets the section, measured inwards from the
 * section's side. The section's corner keeps its radius however little of
 * the section shows: as it folds, the corner rises into the row and melts
 * into it, rather than shrinking to a step. With room below the row for the
 * join and the corner, the join meets the section's straight side; with
 * less, the corner's curve reaches up to the row, and the join is fitted
 * between the row's edge and that curve, touching both. The two meet
 * exactly when the room runs out, so the outline never jumps. */
function joinPoints(
  height: number,
  shown: number,
  { corner, join }: { corner: number; join: number },
) {
  const reach = corner + join;
  if (shown >= reach) {
    return { from: -join, isStraight: true, to: { x: 0, y: height + join } };
  }
  // The join's centre lies a join's radius below the row's edge, a corner
  // and a join's radius from the corner's centre.
  const centreX = corner - Math.sqrt(Math.max(0, shown * (2 * reach - shown)));
  const cornerY = height + shown - corner;
  const toward = corner / reach;
  return {
    from: centreX,
    isStraight: false,
    to: {
      x: corner + (centreX - corner) * toward,
      y: cornerY + (height + join - cornerY) * toward,
    },
  };
}

/** Across the top, from the start side to the end side. Under a notch the
 * top runs into the screen's edge and flares out into it on each side, as
 * the notch's own shoulders do; elsewhere its corners are rounded. */
function topEdge({ flare, top, width }: IslandOutlineShape["row"]) {
  return flare > 0.01
    ? [`M${n(-flare)} 0`, `H${n(width + flare)}`, hollow(flare, width, flare)]
    : [
        `M0 ${n(top)}`,
        round(top, top, 0),
        `H${n(width - top)}`,
        round(top, width, top),
      ];
}

/** Up the start side and back into the top, flaring out under a notch. */
function closeTop({ flare }: IslandOutlineShape["row"]) {
  return flare > 0.01 ? [`V${n(flare)}`, hollow(flare, -flare, 0), "Z"] : ["Z"];
}

/** The curves of a side where the section reaches `out` px past the row's
 * end: the join out of the row's side, the section's corner over the
 * reach, and its corner at the bottom (`corner`, as a side that meets the
 * row's end would have it). Each is held to the room it has, so as the
 * reach closes to nothing they shrink to nothing with it and the side runs
 * straight down, as a meeting side does. */
function reachRadii(
  out: number,
  shown: number,
  { corner, row }: { corner: number; row: IslandOutlineShape["row"] },
) {
  const join = Math.min(
    ISLAND.fillet,
    out / 2,
    Math.max(0, row.height - row.top),
  );
  const top = Math.min(ISLAND.sectionRadius, out - join, shown / 2);
  return { bottom: Math.min(corner, shown - top), join, top };
}

export function islandOutline({ row, section }: IslandOutlineShape): string {
  const { height, width } = row;
  const shown = section ? section.height : 0;
  if (!section || shown < 0.5) {
    const radius = Math.min(row.radius, height / 2, width / 2);
    return [
      ...topEdge(row),
      `V${n(height - radius)}`,
      round(radius, width - radius, height),
      `H${n(radius)}`,
      round(radius, 0, height - radius),
      ...closeTop(row),
    ].join(" ");
  }
  const start = section.start;
  const end = section.start + section.width;
  const bottom = height + shown;
  // How far the section reaches past the row's ends, if it does.
  const startOut = Math.max(0, -start);
  const endOut = Math.max(0, end - width);
  const startSide = sideRadii(Math.max(0, start), shown, row);
  const endSide = sideRadii(width - Math.min(width, end), shown, row);
  // A narrow section cannot hold both its corners at full size.
  const room = (end - start) / 2;
  const startCorner = Math.min(startSide.corner, room);
  const endCorner = Math.min(endSide.corner, room);

  const path = topEdge(row);
  // Down the end side.
  if (endOut > 0) {
    const reach = reachRadii(endOut, shown, { corner: endCorner, row });
    path.push(
      `V${n(height - reach.join)}`,
      hollow(reach.join, width + reach.join, height),
      `H${n(end - reach.top)}`,
      round(reach.top, end, height + reach.top),
      `V${n(bottom - reach.bottom)}`,
      round(reach.bottom, end - reach.bottom, bottom),
    );
  } else if (endSide.isMeeting) {
    path.push(
      `V${n(bottom - endCorner)}`,
      round(endCorner, width - endCorner, bottom),
    );
  } else {
    const join = joinPoints(height, shown, {
      corner: endCorner,
      join: endSide.join,
    });
    path.push(
      `V${n(height - endSide.rowCorner)}`,
      round(endSide.rowCorner, width - endSide.rowCorner, height),
      `H${n(end - join.from)}`,
      hollow(endSide.join, end - join.to.x, join.to.y),
      ...(join.isStraight ? [`V${n(bottom - endCorner)}`] : []),
      round(endCorner, end - endCorner, bottom),
    );
  }
  // Along the bottom and up the start side.
  if (startOut > 0) {
    const reach = reachRadii(startOut, shown, { corner: startCorner, row });
    path.push(
      `H${n(start + reach.bottom)}`,
      round(reach.bottom, start, bottom - reach.bottom),
      `V${n(height + reach.top)}`,
      round(reach.top, start + reach.top, height),
      `H${n(-reach.join)}`,
      hollow(reach.join, 0, height - reach.join),
    );
  } else if (startSide.isMeeting) {
    path.push(
      `H${n(startCorner)}`,
      round(startCorner, 0, bottom - startCorner),
    );
  } else {
    const join = joinPoints(height, shown, {
      corner: startCorner,
      join: startSide.join,
    });
    path.push(
      `H${n(start + startCorner)}`,
      join.isStraight
        ? round(startCorner, start, bottom - startCorner)
        : round(startCorner, start + join.to.x, join.to.y),
      ...(join.isStraight ? [`V${n(join.to.y)}`] : []),
      hollow(startSide.join, start + join.from, height),
      `H${n(startSide.rowCorner)}`,
      round(startSide.rowCorner, 0, height - startSide.rowCorner),
    );
  }
  path.push(...closeTop(row));
  return path.join(" ");
}
