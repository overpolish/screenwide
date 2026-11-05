// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";

/** Whether a picture is being dragged with Option held, so letting go keeps
 * it as a reference (`src-tauri/src/references/drop_island.rs`). */
export function useReferenceDropIsland(): { isUp: boolean } {
  const [isUp, setIsUp] = useState(false);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    // Only macOS watches for the drag; elsewhere there is nothing to ask.
    invoke<boolean>("reference_drop_island_state")
      .then((current) => {
        if (!disposed && !isToldRef.current) setIsUp(current);
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("reference-drop://island", (payload) => {
    isToldRef.current = true;
    setIsUp(payload === true);
  });

  return { isUp };
}
