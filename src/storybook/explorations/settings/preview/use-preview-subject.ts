// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

// A stage follows the row under the pointer or holding focus, so reading
// down the list plays each setting's effect in one place. Leaving a row
// keeps the stage on it: a stage that snapped back to a default whenever the
// pointer crossed a gap between rows would flicker. The pointer is followed
// after a moment, so sweeping across the list does not flash every picture.

const HOVER_DELAY_MS = 120;

/** Which row the stage shows, and the handlers that make a row drive it. */
export function usePreviewSubject<Subject extends string>(initial: Subject) {
  const [subject, setSubject] = useState<Subject>(initial);
  const timerRef = useRef<number | undefined>(undefined);
  useEffect(
    () => () => {
      window.clearTimeout(timerRef.current);
    },
    [],
  );
  const follow = (next: Subject) => ({
    // Pressing a control drives the stage at once, since WebKit does not
    // focus a button it was pressed on.
    onFocus: () => {
      window.clearTimeout(timerRef.current);
      setSubject(next);
    },
    onPointerDown: () => {
      window.clearTimeout(timerRef.current);
      setSubject(next);
    },
    onPointerEnter: () => {
      window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(() => {
        setSubject(next);
      }, HOVER_DELAY_MS);
    },
    onPointerLeave: () => {
      window.clearTimeout(timerRef.current);
    },
  });
  return { follow, subject };
}

export type PreviewFollow = ReturnType<
  ReturnType<typeof usePreviewSubject>["follow"]
>;
