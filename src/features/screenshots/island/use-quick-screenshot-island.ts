// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import { cancelQuickScreenshot, getQuickScreenshotIslandState } from "../api";

import type { QuickScreenshotDestination } from "../../../bindings/QuickScreenshotDestination";

/**
 * A Quick Screenshot session as the island shows it: `isUp` while the
 * overlay is up, and `destination` as last sent, kept after the session
 * ends so the island tucks away saying what it said; `null` until a session
 * has first run.
 */
export function useQuickScreenshotIsland(): {
  destination: QuickScreenshotDestination | null;
  isUp: boolean;
  onClose: () => void;
} {
  const [destination, setDestination] =
    useState<QuickScreenshotDestination | null>(null);
  const [last, setLast] = useState<QuickScreenshotDestination | null>(null);
  if (destination && destination !== last) setLast(destination);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void getQuickScreenshotIslandState().then((current) => {
      if (!disposed && !isToldRef.current) setDestination(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("quick-screenshot://island", (payload) => {
    isToldRef.current = true;
    setDestination(payload as QuickScreenshotDestination | null);
  });

  return {
    destination: destination ?? last,
    isUp: destination !== null,
    onClose: () => {
      cancelQuickScreenshot().catch((cause: unknown) => {
        console.error("Could not cancel the screenshot", cause);
      });
    },
  };
}
