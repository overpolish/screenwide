// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { type ReactNode, useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../lib/use-tauri-event";
import { ISLAND } from "../../primitives/island/island-geometry";

import type { IslandNotch } from "../../primitives/island/island-geometry";
import type { IslandPart } from "../../primitives/island/island-outline-sync";

// The island's window: one, whatever the island shows, as there is one
// notch. Rust brings the island out (`island://shown`, with the notch of the
// screen it is on) and asks it to tuck away (`island://hide`); the window
// goes once the island says it has. Clicks pass through the window
// everywhere but where the island reports its parts to be.

/** What the island's window hands what the island shows. */
export type IslandHost = {
  isShown: boolean;
  notch: IslandNotch | null;
  onHidden: () => void;
  onShapeChange: (parts: IslandPart[]) => void;
};

type Presence = { isShown: boolean; notch: IslandNotch | null };

const reportShape = (parts: IslandPart[]) => {
  void invoke("set_island_hit_area", { rects: parts });
};

const finishHiding = () => {
  void invoke("finish_hiding_island");
};

export function IslandWindow({
  children,
}: {
  children: (island: IslandHost) => ReactNode;
}) {
  const [presence, setPresence] = useState<Presence>({
    isShown: false,
    notch: null,
  });

  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);
  const frameRef = useRef(0);

  useEffect(() => {
    let disposed = false;
    void invoke<Presence>("island_presence").then((current) => {
      if (!disposed && !isToldRef.current) setPresence(current);
    });
    return () => {
      disposed = true;
      cancelAnimationFrame(frameRef.current);
    };
  }, []);

  useTauriEvent("island://shown", (payload) => {
    isToldRef.current = true;
    setPresence((current) => ({
      ...current,
      notch: payload as IslandNotch | null,
    }));
    // Out a frame after the notch it comes out of is laid out, so it is seen
    // coming out of that notch.
    cancelAnimationFrame(frameRef.current);
    frameRef.current = requestAnimationFrame(() => {
      setPresence((current) => ({ ...current, isShown: true }));
    });
  });
  useTauriEvent("island://hide", () => {
    isToldRef.current = true;
    cancelAnimationFrame(frameRef.current);
    setPresence((current) => ({ ...current, isShown: false }));
  });

  return (
    <div className="fixed inset-0">
      <div
        className="absolute inset-x-0"
        style={{ top: presence.notch ? 0 : ISLAND.screenGap }}
      >
        {children({
          isShown: presence.isShown,
          notch: presence.notch,
          onHidden: finishHiding,
          onShapeChange: reportShape,
        })}
      </div>
    </div>
  );
}
