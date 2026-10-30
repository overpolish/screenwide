// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

/** How long a gesture rests, in ms, before its last event is logged. */
const REST_MS = 150;

/**
 * `action`, called with the last arguments once the calls stop for a moment.
 * A drag calls back on every pointer move, and each call logged to Actions
 * is a round trip through Storybook's manager, which makes dragging lag; a
 * story that keeps the drag in its own state logs it through this instead.
 */
export function useRestingAction<TArgs extends unknown[]>(
  action: (...args: TArgs) => void,
) {
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(
    () => () => {
      clearTimeout(timerRef.current);
    },
    [],
  );
  return (...args: TArgs) => {
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      action(...args);
    }, REST_MS);
  };
}
