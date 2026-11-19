// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type RefObject, useLayoutEffect, useRef } from "react";

/** Where the text frame stands in the window, in points: from the window's
 * top, and how wide. */
export type TextFrame = { top: number; width: number };

/** Tells `onChange` where the frame in `frameRef` stands, as it opens and
 * each time it moves or resizes. */
export function useTextFrame(
  frameRef: RefObject<HTMLElement | null>,
  onChange: ((frame: TextFrame) => void) | undefined,
) {
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  useLayoutEffect(() => {
    const frame = frameRef.current;
    if (!frame) return;
    const report = () => {
      const { top, width } = frame.getBoundingClientRect();
      onChangeRef.current?.({ top, width });
    };
    report();
    const observer = new ResizeObserver(report);
    observer.observe(frame);
    return () => {
      observer.disconnect();
    };
  }, [frameRef]);
}
