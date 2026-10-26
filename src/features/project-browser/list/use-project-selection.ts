// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

const NONE: ReadonlySet<string> = new Set();

/**
 * Which listed projects are chosen, by manifest, with the gestures a file
 * browser has: toggle one, extend a range from the last one toggled, all, or
 * none.
 *
 * `order` is what is listed, in the order it is shown, and only the projects
 * that can be chosen. The choice is read through it, so a project hidden by a
 * search or gone from disk is never acted on, however it was chosen.
 */
export function useProjectSelection(order: readonly string[]) {
  const [chosen, setChosen] = useState(NONE);
  const anchorRef = useRef<string | null>(null);
  const selected: ReadonlySet<string> = new Set(
    order.filter((file) => chosen.has(file)),
  );

  return {
    clear: () => {
      anchorRef.current = null;
      setChosen(NONE);
    },
    /** Adds everything from the last project toggled to this one, as a
     * shift-click does; with no such project it toggles this one. */
    extendTo: (file: string) => {
      const from = anchorRef.current ? order.indexOf(anchorRef.current) : -1;
      const to = order.indexOf(file);
      if (to === -1) return;
      if (from === -1) {
        anchorRef.current = file;
        setChosen(new Set([...selected, file]));
        return;
      }
      const range = order.slice(Math.min(from, to), Math.max(from, to) + 1);
      setChosen(new Set([...selected, ...range]));
    },
    selectAll: () => {
      setChosen(new Set(order));
    },
    selected,
    toggle: (file: string) => {
      anchorRef.current = file;
      const next = new Set(selected);
      if (!next.delete(file)) next.add(file);
      setChosen(next);
    },
  };
}
