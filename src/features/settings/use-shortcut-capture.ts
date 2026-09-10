// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef } from "react";

/** Serialises capture begin/end calls and hands the panes a plain
 * `Promise<void>`, so a bridge command that resolves with null still matches
 * the `onCaptureChange` a field expects. */
export function useShortcutCapture(
  begin: () => Promise<unknown>,
  end: () => Promise<unknown>,
) {
  const queueRef = useRef<Promise<void>>(Promise.resolve());
  const countRef = useRef(0);
  return useCallback(
    (capturing: boolean): Promise<void> => {
      countRef.current += capturing ? 1 : -1;
      if (capturing ? countRef.current > 1 : countRef.current > 0)
        return queueRef.current;
      const operation = queueRef.current
        .catch(() => undefined)
        .then(async () => {
          await (capturing ? begin() : end());
        });
      queueRef.current = operation;
      return operation;
    },
    [begin, end],
  );
}
