// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

/** How long the notice offers to undo a delete. Recently Deleted keeps the
 * projects for much longer; this is only the shortcut back. */
const UNDO_WINDOW_MS = 8_000;

type LastDelete = { files: string[]; title: string };

/**
 * The last delete, while its notice can undo it: the projects' manifests
 * in Recently Deleted, which Undo restores.
 */
export function useUndoDelete() {
  const [last, setLast] = useState<LastDelete | null>(null);
  const timerRef = useRef<number | undefined>(undefined);
  useEffect(
    () => () => {
      window.clearTimeout(timerRef.current);
    },
    [],
  );

  return {
    deleted: last ? { count: last.files.length, title: last.title } : null,
    /** Offers to undo a delete of `files`, now in Recently Deleted. */
    record: (files: string[], title: string) => {
      window.clearTimeout(timerRef.current);
      setLast({ files, title });
      timerRef.current = window.setTimeout(() => {
        setLast(null);
      }, UNDO_WINDOW_MS);
    },
    /** The projects to restore, and the notice gone. */
    take: () => {
      window.clearTimeout(timerRef.current);
      setLast(null);
      return last?.files ?? [];
    },
  };
}
