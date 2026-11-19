// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { type ScriptText } from "../script-words";

import { cuesAfterEdit } from "./cue-anchors";
import {
  readCues,
  saveCues,
  type ShownTeleprompterCue,
  TELEPROMPTER_CUE_EVENT,
  TELEPROMPTER_CUE_REDRAWN_EVENT,
} from "./cue-api";
import { type CuePlace, movedTo } from "./place-cue";

/** How long after the last change the cues are saved, in milliseconds. */
const SAVE_AFTER_MS = 500;

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

/**
 * The cues of the script of `project`: read as it opens, saved a moment
 * after each change and as the teleprompter goes, moved with their words
 * as the script is edited, and joined by each cue made for it.
 */
export function useCues(project: string) {
  const [cues, setCues] = useState<ShownTeleprompterCue[]>([]);
  const projectRef = useRef(project);
  projectRef.current = project;
  const pendingRef = useRef<ShownTeleprompterCue[] | null>(null);
  const timerRef = useRef<number | undefined>(undefined);

  const flush = () => {
    window.clearTimeout(timerRef.current);
    const pending = pendingRef.current;
    pendingRef.current = null;
    if (pending)
      saveCues(projectRef.current, pending).catch(
        report("Could not save the cues"),
      );
  };
  const flushRef = useRef(flush);
  flushRef.current = flush;
  const change = (next: ShownTeleprompterCue[]) => {
    setCues(next);
    pendingRef.current = next;
    window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => {
      flushRef.current();
    }, SAVE_AFTER_MS);
  };
  const changeRef = useRef(change);
  changeRef.current = change;
  const cuesRef = useRef(cues);
  cuesRef.current = cues;

  // Read again when the script is renamed, which moves its folder and the
  // pictures in it; anything waiting is saved there first.
  useEffect(() => {
    let disposed = false;
    // A redrawn picture can arrive before the cues are read; it is kept to
    // be swapped in then.
    const redrawn = new Map<string, ShownTeleprompterCue>();
    const withRedrawn = (cue: ShownTeleprompterCue) => {
      const drawn = redrawn.get(cue.id);
      return drawn ? { ...cue, image: drawn.image, path: drawn.path } : cue;
    };
    readCues(project)
      .then((read) => {
        if (disposed) return;
        const cues = read.map(withRedrawn);
        // Redrawn, the cues are kept with their new pictures.
        if (cues.some((cue, index) => cue !== read[index]))
          changeRef.current(cues);
        else setCues(cues);
      })
      .catch(report("Could not read the cues"));
    const unlistening = [
      // A word holds one cue, so a cue made for a word with one replaces it.
      listen<ShownTeleprompterCue>(TELEPROMPTER_CUE_EVENT, ({ payload }) => {
        changeRef.current([
          ...cuesRef.current.filter((cue) => cue.at !== payload.at),
          payload,
        ]);
      }),
      // The cue keeps the word it has moved to since it was read.
      listen<ShownTeleprompterCue>(
        TELEPROMPTER_CUE_REDRAWN_EVENT,
        ({ payload }) => {
          redrawn.set(payload.id, payload);
          if (cuesRef.current.some((cue) => cue.id === payload.id))
            changeRef.current(cuesRef.current.map(withRedrawn));
        },
      ),
    ];
    return () => {
      disposed = true;
      for (const pending of unlistening)
        void pending.then((unlisten) => {
          unlisten();
        });
      flushRef.current();
    };
  }, [project]);

  return {
    cues,
    /** Moves the cues with their words as `before` is edited into
     * `after`, which `script` reads. */
    edited: (before: string, after: string, script: ScriptText) => {
      const moved = cuesAfterEdit({ after, before, script }, cues);
      if (moved.some((cue, index) => cue !== cues[index])) change(moved);
    },
    /** Moves each of `moved` to its word. */
    move: (moved: readonly CuePlace[]) => {
      if (moved.length > 0) change(movedTo(cues, moved));
    },
    remove: (id: string) => {
      change(cues.filter((cue) => cue.id !== id));
    },
  };
}
