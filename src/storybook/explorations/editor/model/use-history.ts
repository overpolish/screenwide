// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useReducer, useRef } from "react";

// Undo and redo over the editor's document: its timing, annotations and
// scenes. A drag sends many changes; changes that follow each other within
// a moment are one step, so a drag undoes in one go.

/** How long, in ms, between changes that makes them separate steps. */
const STEP_GAP_MS = 400;

export function useHistory<Doc>(doc: Doc, restore: (doc: Doc) => void) {
  const pastRef = useRef<Doc[]>([]);
  const futureRef = useRef<Doc[]>([]);
  const lastRef = useRef(doc);
  const changedAtRef = useRef(0);
  // Set while a step is taken back, so the change it makes is not a step.
  const isRestoringRef = useRef(false);
  const [, rerender] = useReducer((count: number) => count + 1, 0);

  useEffect(() => {
    if (lastRef.current === doc) return;
    if (isRestoringRef.current) {
      isRestoringRef.current = false;
    } else {
      const now = performance.now();
      if (now - changedAtRef.current > STEP_GAP_MS) {
        pastRef.current.push(lastRef.current);
        futureRef.current = [];
        rerender();
      }
      changedAtRef.current = now;
    }
    lastRef.current = doc;
  }, [doc]);

  const step = (from: Doc[], to: Doc[]) => {
    const doc = from.pop();
    if (doc === undefined) return;
    to.push(lastRef.current);
    isRestoringRef.current = true;
    changedAtRef.current = 0;
    restore(doc);
    rerender();
  };

  return {
    canRedo: futureRef.current.length > 0,
    canUndo: pastRef.current.length > 0,
    redo: () => {
      step(futureRef.current, pastRef.current);
    },
    undo: () => {
      step(pastRef.current, futureRef.current);
    },
  };
}
