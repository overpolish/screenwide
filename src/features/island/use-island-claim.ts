// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../lib/use-tauri-event";

import type { IslandClaim } from "../../bindings/IslandClaim";

/**
 * The claim on top of the island (`claims.rs`), which is what it shows. Once
 * nothing holds it the last one stays while the island tucks away, so it
 * goes showing what it showed, and is let go once it has (`onHidden`), so
 * the island rests on the recording's, whose page carries the controls'
 * shortcuts. `null` while nothing holds it or has.
 */
export function useIslandClaim(): {
  claim: IslandClaim | null;
  onHidden: () => void;
} {
  const [claims, setClaims] = useState<IslandClaim[]>([]);
  const [last, setLast] = useState<IslandClaim | null>(null);
  const claimsRef = useRef(claims);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  const take = (next: IslandClaim[]) => {
    claimsRef.current = next;
    setClaims(next);
  };

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void invoke<IslandClaim[]>("island_claims").then((current) => {
      if (!disposed && !isToldRef.current) take(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("island://claims", (payload) => {
    isToldRef.current = true;
    take(payload as IslandClaim[]);
  });

  const top = claims.length > 0 ? claims[0] : null;
  if (top && top !== last) setLast(top);
  return {
    claim: top ?? last,
    onHidden: () => {
      if (claimsRef.current.length === 0) setLast(null);
    },
  };
}
