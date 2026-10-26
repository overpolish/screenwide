// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import {
  cancelTextRecognition,
  getTextRecognitionIslandState,
  runTextRecognitionIslandAction,
} from "../api";

import type { TextRecognitionIslandAction } from "../../../bindings/TextRecognitionIslandAction";
import type { TextRecognitionIslandState } from "../../../bindings/TextRecognitionIslandState";

const report = (action: string) => (cause: unknown) => {
  console.error(`Could not ${action}`, cause);
};

/**
 * Text Recognition as the island shows it: `isUp` while it is on the island,
 * and `state` as last sent, kept after it goes so the island tucks away as it
 * was rather than emptied; `null` until it has first been up.
 *
 * The island keeps no state of its own: what it shows is the session, sent
 * whenever it changes.
 */
export function useTextRecognitionIsland(): {
  isUp: boolean;
  onAction: (action: TextRecognitionIslandAction) => void;
  state: TextRecognitionIslandState | null;
} {
  const [state, setState] = useState<TextRecognitionIslandState | null>(null);
  const [last, setLast] = useState<TextRecognitionIslandState | null>(null);
  if (state && state !== last) setLast(state);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void getTextRecognitionIslandState().then((current) => {
      if (!disposed && !isToldRef.current) setState(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("text-recognition://island", (payload) => {
    isToldRef.current = true;
    setState(payload as TextRecognitionIslandState | null);
  });
  // Escape is a global shortcut while Text Recognition is up, handed to this
  // window, which is up whatever the island shows.
  useTauriEvent("text-recognition://dismiss-requested", () => {
    cancelTextRecognition().catch(report("close Text Recognition"));
  });

  return {
    isUp: state !== null,
    onAction: (action) => {
      runTextRecognitionIslandAction(action).catch(
        report(`run Text Recognition's ${action}`),
      );
    },
    state: state ?? last,
  };
}
