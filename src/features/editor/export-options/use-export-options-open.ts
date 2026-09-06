// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

import { EditorKind } from "../types";

import {
  listenToExportOptionsClosed,
  listenToExportOptionsOpened,
  showExportOptions,
} from "./api";

/**
 * Whether this editor's options window is up, and how to put it up.
 *
 * Rust announces both edges, because the window closes by routes the editor
 * never sees: Escape, its own close button, Cancel, Export, or the workspace
 * being stood down. Opening is also taken optimistically, so the editor dims
 * on the click rather than a round trip later.
 */
export function useExportOptionsOpen(kind: EditorKind) {
  const [isExportOpen, setIsExportOpen] = useState(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    const listeners: UnlistenFn[] = [];
    const track = async () => {
      const opened = await listenToExportOptionsOpened((opening) => {
        if (opening === kind) setIsExportOpen(true);
      });
      const closed = await listenToExportOptionsClosed((closing) => {
        if (closing === kind) setIsExportOpen(false);
      });
      listeners.push(opened, closed);
      if (disposed) for (const dispose of listeners) dispose();
    };
    void track();
    return () => {
      disposed = true;
      for (const dispose of listeners) dispose();
    };
  }, [kind]);

  const open = useCallback(() => {
    setIsExportOpen(true);
    showExportOptions().catch((cause: unknown) => {
      console.error("Could not open the export options", cause);
      setIsExportOpen(false);
    });
  }, []);

  return { isExportOpen, open };
}
