// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useReducer } from "react";

import type { NotesDocument } from "../document/notes-document";

// The notes as they were, for Undo and Redo. A run of typing in one turn is
// one step, as a text editor's is: changes with the same `typing` key
// within a moment of each other fold into the step before.

/** How long a pause in typing ends a step, in milliseconds. */
const TYPING_STEP_MS = 1_000;
/** How many steps back Undo reaches. */
const STEPS = 200;

type History = {
  future: NotesDocument[];
  lastTyping: { at: number; key: string } | null;
  past: NotesDocument[];
  present: NotesDocument | null;
};

type Action =
  | { document: NotesDocument; type: "change"; typing?: string }
  | { document: NotesDocument | null; type: "reset" }
  | { title: string; type: "retitle" }
  | { type: "redo" }
  | { type: "undo" };

/** `document` under `title`. */
const titled = (document: NotesDocument, title: string) => ({
  ...document,
  title,
});

function reduce(history: History, action: Action): History {
  switch (action.type) {
    case "reset":
      return {
        future: [],
        lastTyping: null,
        past: [],
        present: action.document,
      };
    // The name is the project's, not an edit to undo: every step takes it.
    case "retitle":
      return {
        ...history,
        future: history.future.map((step) => titled(step, action.title)),
        past: history.past.map((step) => titled(step, action.title)),
        present: history.present && titled(history.present, action.title),
      };
    case "change": {
      const { present } = history;
      if (!present) return { ...history, present: action.document };
      const now = Date.now();
      const folds =
        action.typing !== undefined &&
        history.lastTyping?.key === action.typing &&
        now - history.lastTyping.at < TYPING_STEP_MS;
      return {
        future: [],
        lastTyping:
          action.typing === undefined ? null : { at: now, key: action.typing },
        past: folds ? history.past : [...history.past, present].slice(-STEPS),
        present: action.document,
      };
    }
    case "undo": {
      if (history.past.length === 0 || !history.present) return history;
      const previous = history.past[history.past.length - 1];
      return {
        future: [history.present, ...history.future],
        lastTyping: null,
        past: history.past.slice(0, -1),
        present: previous,
      };
    }
    case "redo": {
      if (history.future.length === 0 || !history.present) return history;
      const [next, ...future] = history.future;
      return {
        future,
        lastTyping: null,
        past: [...history.past, history.present],
        present: next,
      };
    }
  }
}

/** The notes being edited, with Undo and Redo over what was done to them.
 * `reset` starts again from notes read afresh, with nothing to undo. The
 * actions stay the same from render to render. */
export function useNotesHistory() {
  const [history, dispatch] = useReducer(reduce, {
    future: [],
    lastTyping: null,
    past: [],
    present: null,
  });
  const actions = useMemo(
    () => ({
      change: (document: NotesDocument, typing?: string) => {
        dispatch({ document, type: "change", typing });
      },
      redo: () => {
        dispatch({ type: "redo" });
      },
      reset: (document: NotesDocument | null) => {
        dispatch({ document, type: "reset" });
      },
      retitle: (title: string) => {
        dispatch({ title, type: "retitle" });
      },
      undo: () => {
        dispatch({ type: "undo" });
      },
    }),
    [],
  );
  return {
    ...actions,
    canRedo: history.future.length > 0,
    canUndo: history.past.length > 0,
    document: history.present,
  };
}
