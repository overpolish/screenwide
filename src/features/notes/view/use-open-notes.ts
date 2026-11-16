// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { OPEN_NOTES_EVENT, takePendingNotes } from "../api";

/** The notes project the projects window shows in place of its projects, by
 * its manifest, and how to show another or none: asked for once the page has
 * loaded, in case the window was made to show them, then told. */
export function useOpenNotes(): [
  string | null,
  (project: string | null) => void,
] {
  const [project, setProject] = useState<string | null>(null);
  useEffect(() => {
    if (!isTauri()) return;
    void takePendingNotes().then((pending) => {
      if (pending) setProject(pending);
    });
    const unlisten = listen<string>(OPEN_NOTES_EVENT, ({ payload }) => {
      void takePendingNotes();
      setProject(payload);
    });
    return () => {
      void unlisten.then((stop) => {
        stop();
      });
    };
  }, []);
  return [project, setProject];
}
