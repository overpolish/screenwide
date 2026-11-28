// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import { decibelLevel } from "../../../primitives/audio-meter/decibel-level";
import { getDictationIslandState, runDictationIslandAction } from "../api";

import type { DictationIslandAction } from "../../../bindings/DictationIslandAction";
import type { DictationIslandState } from "../../../bindings/DictationIslandState";

/**
 * Dictation as the island shows it: `state` as last sent, kept after it
 * goes so the island tucks away as it was rather than emptied; `null` until
 * it has first been up. `level` is the
 * microphone's, 0 to 1.
 */
export function useDictationIsland(): {
  level: number;
  onAction: (action: DictationIslandAction) => void;
  state: DictationIslandState | null;
} {
  const [state, setState] = useState<DictationIslandState | null>(null);
  const [last, setLast] = useState<DictationIslandState | null>(null);
  if (state && state !== last) setLast(state);
  const [level, setLevel] = useState(0);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void getDictationIslandState().then((current) => {
      if (!disposed && !isToldRef.current) setState(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("dictation://island", (payload) => {
    isToldRef.current = true;
    setState(payload as DictationIslandState | null);
  });
  useTauriEvent("dictation://level", (payload) => {
    setLevel(decibelLevel(payload as number));
  });

  return {
    level: state?.phase === "listening" ? level : 0,
    onAction: (action) => {
      runDictationIslandAction(action).catch((cause: unknown) => {
        console.error(`Could not run dictation's ${action}`, cause);
      });
    },
    state: state ?? last,
  };
}
