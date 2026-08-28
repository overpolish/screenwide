// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef } from "react";

export function useMeasuredHeight<T extends HTMLElement>(
  onHeightChange: (height: number) => void,
) {
  const elementRef = useRef<T>(null);
  const lastHeightRef = useRef<number | null>(null);

  useLayoutEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    const reportHeight = () => {
      const height = Math.ceil(element.getBoundingClientRect().height);
      if (height <= 0 || height === lastHeightRef.current) return;
      lastHeightRef.current = height;
      onHeightChange(height);
    };
    const observer = new ResizeObserver(reportHeight);
    observer.observe(element);
    reportHeight();

    return () => {
      observer.disconnect();
    };
  }, [onHeightChange]);

  return elementRef;
}
