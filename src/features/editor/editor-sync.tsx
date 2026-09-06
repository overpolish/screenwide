// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { getEditorSnapshot } from "./api";
import { useEditorStore } from "./store";
import { EditorSnapshot } from "./types";

const EDITOR_CHANGED_EVENT = "editor://artifact";

export function EditorSync() {
  const setSnapshot = useEditorStore((state) => state.setSnapshot);
  const setSnapshots = useEditorStore((state) => state.setSnapshots);

  useEffect(() => {
    if (!isTauri()) return;

    let disposed = false;
    let unlisten: UnlistenFn | undefined;

    const synchronize = async () => {
      unlisten = await listen<EditorSnapshot>(
        EDITOR_CHANGED_EVENT,
        ({ payload }) => {
          setSnapshot(payload);
        },
      );

      if (disposed) {
        unlisten();
        return;
      }

      // Every workspace at once: a webview that has just come up missed
      // whatever change events landed before it was listening.
      setSnapshots(await getEditorSnapshot());
    };

    void synchronize();

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [setSnapshot, setSnapshots]);

  return null;
}
