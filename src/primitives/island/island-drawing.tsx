// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RefObject } from "react";

import { cn } from "../../lib/styling";

import { ISLAND, px } from "./island-geometry";

// The island's outline, under everything it holds (`useIslandOutline` draws
// it and sizes its box). It carries the shadow itself, so content changing,
// as the recording's time does each second, does not redraw it. Mirrored
// right to left; its box leaves room for the shadow, drawn only inside it.
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
        height: px(room * 2),
        insetInlineStart: -room,
        top: -room,
        width: `calc(100% + ${px(room * 2)})`,
      }}
    >
      {/* Marked so a picture of the island can measure the black it draws
          (`StageScreen`). */}
      <g
        data-island-outline
        transform={`translate(${String(room)} ${String(room)})`}
      >
        <path ref={rowPathRef} />
        <path ref={shapePathRef} />
      </g>
    </svg>
  );
}
