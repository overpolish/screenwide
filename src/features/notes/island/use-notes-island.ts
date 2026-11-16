// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import { getNotesIslandState, runNotesIslandAction } from "../api";

import type { NotesIslandAction } from "../../../bindings/NotesIslandAction";
import type { NotesIslandState } from "../../../bindings/NotesIslandState";

/**
 * Notes as the island shows them: `isUp` while they are on the island, and
 * `state` as last sent, kept after it goes so the island tucks away as it
 * was; `null` until it has first been up.
 */
export function useNotesIsland(): {
  isUp: boolean;
  onAction: (action: NotesIslandAction) => void;
  state: NotesIslandState | null;
} {
  const [state, setState] = useState<NotesIslandState | null>(null);
  const [last, setLast] = useState<NotesIslandState | null>(null);
  if (state && state !== last) setLast(state);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void getNotesIslandState().then((current) => {
      if (!disposed && !isToldRef.current) setState(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("notes://island", (payload) => {
    isToldRef.current = true;
    setState(payload as NotesIslandState | null);
  });

  return {
    isUp: state !== null,
    onAction: (action) => {
      runNotesIslandAction(action).catch((cause: unknown) => {
        console.error(`Could not run the notes' ${action}`, cause);
      });
    },
    state: state ?? last,
  };
}
