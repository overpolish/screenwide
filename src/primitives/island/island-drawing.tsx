// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RefObject } from "react";

import { cn } from "../../lib/styling";

import { ISLAND, px } from "./island-geometry";

/** Taller than the island will grow, for the outline's box. */
const BOX_HEIGHT = 640;

/** The island's outline, under everything it holds (`useIslandOutline`
 * draws it). It carries the shadow itself, so what the island holds
 * changing, as the recording's time does each second, does not redraw it.
 * Drawn from the row's start, it is mirrored right to left. Its box has
 * room round the island for the shadow, which is drawn only inside the
 * box. */
export function IslandDrawing({
  isShown,
  rowPathRef,
  shapePathRef,
}: {
  isShown: boolean;
  rowPathRef: RefObject<SVGPathElement | null>;
  shapePathRef: RefObject<SVGPathElement | null>;
}) {
  const room = ISLAND.shadowRoom;
  return (
    <svg
      aria-hidden
      className={cn(
        "pointer-events-none absolute overflow-visible fill-island rtl:-scale-x-100",
        isShown && "filter-(--island-shadow)",
      )}
      style={{
        height: px(BOX_HEIGHT + room * 2),
        insetInlineStart: -room,
        top: -room,
        width: `calc(100% + ${px(room * 2)})`,
      }}
    >
      <g transform={`translate(${String(room)} ${String(room)})`}>
        <path ref={rowPathRef} />
        <path ref={shapePathRef} />
      </g>
    </svg>
  );
}
