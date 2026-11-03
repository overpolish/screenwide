// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import { cancelScrollingCapture, getScrollingCaptureIslandState } from "../api";

import type { ScrollingCaptureIslandState } from "../../../bindings/ScrollingCaptureIslandState";

/**
 * A scrolling capture as the island shows it: `isUp` while one runs, and
 * `state` as last sent, kept after it ends so the island tucks away saying
 * it is finishing rather than emptied; `null` until one has first run.
 */
export function useScrollingCaptureIsland(): {
  isUp: boolean;
  onCancel: () => void;
  state: ScrollingCaptureIslandState | null;
} {
  const [state, setState] = useState<ScrollingCaptureIslandState | null>(null);
  const [last, setLast] = useState<ScrollingCaptureIslandState | null>(null);
  if (state && state !== last) setLast(state);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void getScrollingCaptureIslandState().then((current) => {
      if (!disposed && !isToldRef.current) setState(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("scrolling-capture://island", (payload) => {
    isToldRef.current = true;
    setState(payload as ScrollingCaptureIslandState | null);
  });

  return {
    isUp: state !== null,
    onCancel: () => {
      cancelScrollingCapture().catch((cause: unknown) => {
        console.error("Could not cancel the scrolling capture", cause);
      });
    },
    state: state ?? last,
  };
}
