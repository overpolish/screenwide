// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, useLayoutEffect, useRef, useState } from "react";

/**
 * How wide the tools in a title bar may grow while staying on the window's
 * centre line, in CSS px, or null outside a measured header. Tools that would
 * not fit read it to decide what to fold away.
 */
export const WindowHeaderCenterWidthContext = createContext<number | null>(
  null,
);

/**
 * Measures the room the centre column has: the bar's width less two copies of
 * the wider side, since the centre stays centred only while both sides are
 * equal. The leading side counts at its minimum, so the title gives way down
 * to it; the trailing side counts at its content width. Neither depends on the
 * tools, so the measurement does not chase its own result.
 */
export function useWindowHeaderCenterWidth(isEnabled: boolean) {
  const headerRef = useRef<HTMLElement>(null);
  const leadingRef = useRef<HTMLDivElement>(null);
  const trailingRef = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState<number | null>(null);

  useLayoutEffect(() => {
    const header = headerRef.current;
    if (!isEnabled || !header) return;
    const measure = () => {
      const style = getComputedStyle(header);
      const inner =
        header.clientWidth -
        parseFloat(style.paddingLeft) -
        parseFloat(style.paddingRight);
      const gap = parseFloat(style.columnGap) || 0;
      const leading = leadingRef.current
        ? parseFloat(getComputedStyle(leadingRef.current).minWidth) || 0
        : 0;
      const trailing = trailingRef.current?.getBoundingClientRect().width ?? 0;
      setWidth(
        Math.max(
          0,
          Math.floor(inner - 2 * Math.max(leading, trailing) - 2 * gap),
        ),
      );
    };
    // An observer reports each target once as it starts watching, before the
    // frame is painted, so this is also the first measurement.
    const observer = new ResizeObserver(measure);
    observer.observe(header);
    if (trailingRef.current) observer.observe(trailingRef.current);
    return () => {
      observer.disconnect();
    };
  }, [isEnabled]);

  return {
    headerRef,
    leadingRef,
    trailingRef,
    width: isEnabled ? width : null,
  };
}
