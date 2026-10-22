// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useState } from "react";

type Size = { height: number; width: number };

/** An element's laid-out size, kept up to date as it changes. A new element
 * has no size until it is measured, so nothing is placed by what was there
 * before it; once the element is gone, its last size stays, so what was
 * there can fold away from it. Read after every render before paint, so a
 * size that changes with what it holds is known in the frame it is drawn, as
 * the island's shift is; an observer alone answers a frame late. The
 * observer catches the rest. */
export function useSize(element: HTMLElement | null) {
  const [measured, setMeasured] = useState<{
    element: HTMLElement;
    size: Size;
  } | null>(null);
  // Every render, deliberately: the size is read where it changes. It cannot
  // loop, as an unchanged size keeps the same state and React stops there.
  // eslint-disable-next-line @eslint-react/exhaustive-deps
  useLayoutEffect(() => {
    if (!element) return;
    const height = element.offsetHeight;
    const width = element.offsetWidth;
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setMeasured((current) =>
      current?.element === element &&
      current.size.height === height &&
      current.size.width === width
        ? current
        : { element, size: { height, width } },
    );
  });
  useLayoutEffect(() => {
    if (!element) return;
    const observer = new ResizeObserver(() => {
      setMeasured({
        element,
        size: { height: element.offsetHeight, width: element.offsetWidth },
      });
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, [element]);
  if (!element) return measured?.size ?? null;
  return measured?.element === element ? measured.size : null;
}
