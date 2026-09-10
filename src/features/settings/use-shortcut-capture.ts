// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef } from "react";

export function useShortcutCapture(
  begin: () => Promise<null>,
  end: () => Promise<null>,
) {
  const queueRef = useRef(Promise.resolve());
  const countRef = useRef(0);
  return useCallback(
    (capturing: boolean) => {
      countRef.current += capturing ? 1 : -1;
      if (capturing ? countRef.current > 1 : countRef.current > 0)
        return queueRef.current;
      const operation = queueRef.current
        .catch(() => undefined)
        .then(() => (capturing ? begin() : end()));
      queueRef.current = operation;
      return operation;
    },
    [begin, end],
  );
}
