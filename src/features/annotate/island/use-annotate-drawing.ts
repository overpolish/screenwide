// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import { isAnnotateDrawing, returnAnnotateKeyboard } from "../api";

/**
 * Whether the overlay is drawing, which is when the island carries the
 * annotate tools.
 *
 * While it is, every press on the island gives the keyboard back to the
 * picture once released. On Windows the island refuses activation but not
 * focus, so a press moves the keyboard into its page, where the letters that
 * pick tools up go unheard; on macOS the hand-back finds nothing to move.
 */
export function useAnnotateDrawing() {
  const [isDrawing, setIsDrawing] = useState(false);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void isAnnotateDrawing().then((current) => {
      if (!disposed && !isToldRef.current) setIsDrawing(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("annotate://drawing", (payload) => {
    isToldRef.current = true;
    setIsDrawing(payload === true);
  });

  useEffect(() => {
    if (!isDrawing) return;
    const giveBack = () => {
      returnAnnotateKeyboard().catch(() => undefined);
    };
    document.addEventListener("pointerup", giveBack);
    return () => {
      document.removeEventListener("pointerup", giveBack);
    };
  }, [isDrawing]);

  return isDrawing;
}
