// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef, useState } from "react";

/**
 * How many pens fit in the tool column, measured rather than worked out
 * from sizes, so it follows the buttons and the modes however they are
 * drawn. The pens take what the column's height leaves once the modes have
 * theirs; Select, first among them, always shows. Null while every pen
 * fits, which `annotationToolLayout` reads as no overflow.
 *
 * `count` is how many pens there are, Select not among them.
 */
export function usePenCapacity(count: number) {
  const columnRef = useRef<HTMLDivElement>(null);
  const modesRef = useRef<HTMLDivElement>(null);
  const toolsRef = useRef<HTMLDivElement>(null);
  const [capacity, setCapacity] = useState<number | null>(null);

  useLayoutEffect(() => {
    const column = columnRef.current;
    const modes = modesRef.current;
    const tools = toolsRef.current;
    if (!column || !modes || !tools) return;
    const measure = () => {
      const button = tools.querySelector("button")?.offsetHeight;
      const toolbar = tools.firstElementChild;
      if (!button || !toolbar) return;
      const gap = parseFloat(getComputedStyle(toolbar).rowGap) || 0;
      const room = column.clientHeight - modes.offsetHeight;
      // Every button takes its height and one gap; the last gap is not
      // drawn, so the room is given it back. Select takes the first place.
      const pens = Math.floor((room + gap) / (button + gap)) - 1;
      setCapacity(pens >= count ? null : Math.max(0, pens));
    };
    // An observer reports each element's size as soon as it watches it,
    // before the frame is painted, so the first measure needs no call here.
    const observer = new ResizeObserver(measure);
    observer.observe(column);
    observer.observe(modes);
    return () => {
      observer.disconnect();
    };
  }, [count]);

  return { capacity, columnRef, modesRef, toolsRef };
}
