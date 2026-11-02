// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import type { DisplayNotches } from "../../../bindings/DisplayNotches";

// Whether this machine's displays have a notch, so a preview of the island
// takes the notch's layout only where the island would: the display it
// comes out on, and the main display. No notch until the answer comes, on
// Windows, and outside the app.

const NONE: DisplayNotches = { island: null, main: null };

export function useDisplayNotches() {
  const [notches, setNotches] = useState(NONE);

  useEffect(() => {
    if (!isTauri()) return;
    let current = true;
    void invoke<DisplayNotches>("display_notches").then(
      (found) => {
        if (current) setNotches(found);
      },
      () => {
        // No notch is the safe layout, and stays.
      },
    );
    return () => {
      current = false;
    };
  }, []);

  return notches;
}
