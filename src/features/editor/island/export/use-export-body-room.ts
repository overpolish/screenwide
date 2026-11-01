// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RefObject, useLayoutEffect, useState } from "react";

import { motionDurations } from "../../../../lib/motion";
import { ISLAND } from "../../../../primitives/island/island-geometry";
import { ISLAND_ROOM, ISLAND_ROOM_HOLDER } from "../island-room";

/** How far `element`'s top lies below `holder`'s in layout, which the
 * island's transforms leave alone as it changes shape. */
const layoutTop = (element: HTMLElement, holder: Element) => {
  let top = 0;
  for (
    let at: Element | null = element;
    at instanceof HTMLElement && at !== holder;
    at = at.offsetParent
  ) {
    top += at.offsetTop;
  }
  return top;
};

/**
 * The most the Export island's body may be tall, as a CSS length: the room
 * its window leaves the island (`island-room.ts`) less all that stands above
 * the body (the row, the file line) and below it down to the island's foot
 * (Export and the section's inset), so in a short window the body scrolls
 * while the row and Export stay in view. `body` is the part that scrolls and
 * `column` the part it heads, whose foot is the section's. Undefined until
 * measured, or where nothing gives the island a room.
 *
 * What stands round the body does not change with its height, so cutting
 * it short never changes what is measured. The section slides under the
 * row as the island comes out, so it is measured again once that is done.
 */
export function useExportBodyRoom(
  body: RefObject<HTMLElement | null>,
  column: RefObject<HTMLElement | null>,
) {
  const [rest, setRest] = useState<number | null>(null);
  useLayoutEffect(() => {
    const scrolled = body.current;
    const end = column.current;
    const holder = scrolled?.closest(ISLAND_ROOM_HOLDER);
    if (!scrolled || !end || !holder) return;
    let timer = 0;
    const measure = () => {
      // The island's foot less the body is all that stands above and below
      // it.
      const foot = layoutTop(end, holder) + end.offsetHeight;
      setRest(foot - scrolled.offsetHeight + ISLAND.sectionInset);
    };
    const observer = new ResizeObserver(() => {
      measure();
      window.clearTimeout(timer);
      timer = window.setTimeout(measure, motionDurations.travel * 1000);
    });
    observer.observe(end);
    return () => {
      observer.disconnect();
      window.clearTimeout(timer);
    };
  }, [body, column]);
  return rest === null
    ? undefined
    : `calc(var(${ISLAND_ROOM}) - ${String(rest)}px)`;
}
