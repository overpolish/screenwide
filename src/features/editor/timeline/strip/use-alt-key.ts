// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

/** Whether ⌥ (Alt) is held. On the strip ⌥ is the scissors: held, a drag
 * cuts rather than scrubs or retimes, and the strip shows it before
 * anything is pressed. */
export function useAltKey() {
  const [isAlt, setIsAlt] = useState(false);
  useEffect(() => {
    const read = (event: KeyboardEvent) => {
      setIsAlt(event.altKey);
    };
    // Switching away with ⌥ down never sends its key up.
    const release = () => {
      setIsAlt(false);
    };
    window.addEventListener("keydown", read);
    window.addEventListener("keyup", read);
    window.addEventListener("blur", release);
    return () => {
      window.removeEventListener("keydown", read);
      window.removeEventListener("keyup", read);
      window.removeEventListener("blur", release);
    };
  }, []);
  return isAlt;
}
