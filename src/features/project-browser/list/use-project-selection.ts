// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import type { ProjectActions } from "./use-project-menu";

const NONE: ReadonlySet<string> = new Set();

/**
 * The actions, with the choice ending as projects are deleted, moved,
 * restored or sent to the Trash, as a file browser's does. Those projects
 * leave the list, and a choice kept for them comes back whenever the list
 * shows them again, as it can for a moment while it reloads, bringing the
 * island's selection row back over the notice of what was done.
 */
export function clearingChoice(
  actions: ProjectActions,
  clear: () => void,
): ProjectActions {
  const then =
    <Args extends unknown[]>(act: (...args: Args) => void) =>
    (...args: Args) => {
      clear();
      act(...args);
    };
  if ("onRestore" in actions)
    return {
      ...actions,
      onRestore: then(actions.onRestore),
      onTrash: then(actions.onTrash),
    };
  return {
    ...actions,
    onDelete: then(actions.onDelete),
    onMove: actions.onMove && then(actions.onMove),
  };
}

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
