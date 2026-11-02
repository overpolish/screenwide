// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

// A pane's picture follows the row under the pointer or holding focus, so
// reading down the list plays each setting's effect in one place. Leaving a
// row keeps the picture on it: one that snapped back to a default whenever
// the pointer crossed a gap between rows would flicker. The pointer is
// followed after a moment, so sweeping across the list does not flash every
// picture.

const HOVER_DELAY_MS = 120;

/** What a row spreads on itself to drive the picture. */
export type PreviewFollow = {
  onFocus: () => void;
  onPointerDown: () => void;
  onPointerEnter: () => void;
  onPointerLeave: () => void;
};

/** Which row the picture shows, and the handlers that make a row drive
 * it. */
export function usePreviewSubject<Subject extends string>(initial: Subject) {
  const [subject, setSubject] = useState<Subject>(initial);
  const timerRef = useRef<number | undefined>(undefined);
  useEffect(
    () => () => {
      window.clearTimeout(timerRef.current);
    },
    [],
  );
  const show = (next: Subject) => {
    window.clearTimeout(timerRef.current);
    setSubject(next);
  };
  const follow = (next: Subject): PreviewFollow => ({
    // Focus reaches here from any control in the row. Pressing drives the
    // picture at once too, since WebKit does not focus a button it was
    // pressed on.
    onFocus: () => {
      show(next);
    },
    onPointerDown: () => {
      show(next);
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
