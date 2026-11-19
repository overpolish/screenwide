// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect, useRef, useState } from "react";

import { saveScript } from "./api";

/** How long after the last change the script is saved, in milliseconds. */
const SAVE_AFTER_MS = 500;

/**
 * Saves the script of `project` a moment after the last change, and as the
 * teleprompter goes. What waits is saved to wherever the project is when
 * it is written, so a rename in between does not lose it.
 */
export function useScriptSaving(project: string) {
  const [error, setError] = useState<string>();
  const projectRef = useRef(project);
  projectRef.current = project;
  const pendingRef = useRef<string | null>(null);
  const timerRef = useRef<number | undefined>(undefined);

  const flush = () => {
    window.clearTimeout(timerRef.current);
    const pending = pendingRef.current;
    pendingRef.current = null;
    if (pending === null) return Promise.resolve();
    return saveScript(projectRef.current, pending).then(
      () => {
        setError(undefined);
      },
      (cause: unknown) => {
        setError(String(cause));
      },
    );
  };
  const flushRef = useRef(flush);
  flushRef.current = flush;
  useEffect(
    () => () => {
      void flushRef.current();
    },
    [],
  );

  return {
    error,
    flush,
    /** Saves `markdown` once the changes stop for a moment. */
    save: (markdown: string) => {
      pendingRef.current = markdown;
      window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(() => {
        void flushRef.current();
      }, SAVE_AFTER_MS);
    },
  };
}
