// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useState } from "react";

// The spring's badge sits on the top edge of its box, and its fill is
// translucent, so what lies under it, the box's fill and its outline, fades
// away round it and the badge reads as standing over them. The fade is a
// pill the badge's shape, blurred, so it feathers out as a scroll area's
// edge does rather than leaving a hard cut.

/** The room, in px, left round the badge before the fade begins. */
const AIR_PX = 1;
/** How far, in px, the box fades back in past that room. */
const FEATHER_PX = 5;

export type BadgeSize = { height: number; width: number };

/** The badge's size as drawn, measured, since its words change with the
 * speed as an end is dragged; null until it is drawn. */
export function useBadgeSize() {
  const [size, setSize] = useState<BadgeSize | null>(null);
  const ref = useCallback((element: HTMLSpanElement | null) => {
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      const [box] = entry.borderBoxSize;
      setSize({ height: box.blockSize, width: box.inlineSize });
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);
  return { ref, size };
}

/** A mask fading the badge's room out of the top middle of a surface whose
 * top edge sits `above` px over the box's, where the badge is centred. */
export const notched = (size: BadgeSize, above = 0) => {
  const pill = {
    height: size.height + AIR_PX * 2,
    width: size.width + AIR_PX * 2,
  };
  // Only the part of the pill on the surface is drawn; the blur reaches a
  // feather's width past the pill on every side.
  const width = pill.width + FEATHER_PX * 2;
  const height = above + pill.height / 2 + FEATHER_PX;
  const svg = [
    `<svg xmlns='http://www.w3.org/2000/svg' width='${String(width)}' height='${String(height)}'>`,
    `<filter id='f' x='-50%' y='-50%' width='200%' height='200%'><feGaussianBlur stdDeviation='${String(FEATHER_PX / 2)}'/></filter>`,
    `<rect filter='url(#f)' x='${String(FEATHER_PX)}' y='${String(above - pill.height / 2)}' width='${String(pill.width)}' height='${String(pill.height)}' rx='${String(pill.height / 2)}'/>`,
    `</svg>`,
  ].join("");
  return {
    maskComposite: "exclude",
    maskImage: `linear-gradient(#000, #000), url("data:image/svg+xml,${encodeURIComponent(svg)}")`,
    maskPosition: "0 0, 50% 0",
    maskRepeat: "no-repeat",
    maskSize: `100% 100%, ${String(width)}px ${String(height)}px`,
  };
};
