// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useRef, useState } from "react";

/**
 * A text value that is edited here but owned by another window.
 *
 * Every edit is shown at once and sent onward; the owner's echo comes back a
 * moment later through the mirror. An echo that matches something still in
 * flight is dropped rather than adopted, so a fast second keystroke is never
 * overwritten by the echo of the first. Anything else from the owner, such as
 * a new artifact's name, replaces the draft.
 */
export function useEchoedValue(
  published: string,
  send: (value: string) => void,
) {
  const [draft, setDraft] = useState(published);
  const inFlightRef = useRef<string[]>([]);

  useEffect(() => {
    const index = inFlightRef.current.indexOf(published);
    if (index === -1) {
      setDraft(published);
      return;
    }
    inFlightRef.current.splice(0, index + 1);
  }, [published]);

  const change = useCallback(
    (value: string) => {
      setDraft(value);
      inFlightRef.current.push(value);
      send(value);
    },
    [send],
  );

  return [draft, change] as const;
}
