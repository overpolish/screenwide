// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef, useState } from "react";

/**
 * How many pens fit in the tool column, measured rather than worked out
 * from sizes, so it follows the buttons however they are drawn. The pens
 * take the column's height; Select and Crop, first among them, always
 * show. Null while every pen fits, which `annotationToolLayout` reads as no
 * overflow.
 *
 * `count` is how many pens there are, Select and Crop not among them.
 */
export function usePenCapacity(count: number) {
  const columnRef = useRef<HTMLDivElement>(null);
  const toolsRef = useRef<HTMLDivElement>(null);
  const [capacity, setCapacity] = useState<number | null>(null);

  useLayoutEffect(() => {
    const column = columnRef.current;
    const tools = toolsRef.current;
    if (!column || !tools) return;
    const measure = () => {
      const button = tools.querySelector("button")?.offsetHeight;
      const toolbar = tools.firstElementChild;
      if (!button || !toolbar) return;
      const gap = parseFloat(getComputedStyle(toolbar).rowGap) || 0;
      // Every button takes its height and one gap; the last gap is not
      // drawn, so the room is given it back. Select and Crop take the first
      // places.
      const pens = Math.floor((column.clientHeight + gap) / (button + gap)) - 2;
      setCapacity(pens >= count ? null : Math.max(0, pens));
    };
    // An observer reports each element's size as soon as it watches it,
    // before the frame is painted, so the first measure needs no call here.
    const observer = new ResizeObserver(measure);
    observer.observe(column);
    return () => {
      observer.disconnect();
    };
  }, [count]);

  return { capacity, columnRef, toolsRef };
}
