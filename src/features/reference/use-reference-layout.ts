// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useInstantLayoutTransition } from "motion/react";
import { useEffect, useRef, useState } from "react";

import type { ReferenceLayout } from "../../bindings/ReferenceLayout";

// Where a reference's island stands. The picture is drawn natively, and the
// window is placed round it, the island and the room for their shadows in
// Rust (`references/fit.rs`); the page says how big the island is and takes
// back where it stands. Rust tells the page itself as the picture is moved
// or resized, and the island goes over the picture near the screen's foot.
// The window is fitted again as the island changes size, and once it has
// come to rest after the arrow keys move it.

/** How long the window must be still after a nudge before it is fitted
 * again, in ms. */
const SETTLE_MS = 250;

/** The event Rust tells the page where the island stands by
 * (`references/macos.rs`). */
const LAYOUT_EVENT = "reference-layout";

interface Placed {
  /** Where the island stood before it went to the other side of the
   * picture with this layout; null where it stays on its side. */
  flippedFrom: null | number;
  layout: null | ReferenceLayout;
}

export function useReferenceLayout() {
  const [placed, setPlaced] = useState<Placed>({
    flippedFrom: null,
    layout: null,
  });
  /** The island at its own size. */
  const islandRef = useRef<HTMLDivElement>(null);
  const instantly = useInstantLayoutTransition();

  useEffect(() => {
    const island = islandRef.current;
    if (!island) return;
    const host = getCurrentWebviewWindow();
    // The island moving as a whole is no change to its parts, which would
    // otherwise glide after it from where they were drawn (`IslandParts`).
    const take = (layout: ReferenceLayout) => {
      instantly(() => {
        setPlaced(({ layout: before }) => ({
          flippedFrom:
            before !== null && before.islandAbove !== layout.islandAbove
              ? before.islandY
              : null,
          layout,
        }));
      });
    };
    const fit = () => {
      invoke<ReferenceLayout>("fit_reference", {
        height: island.offsetHeight,
        width: island.offsetWidth,
      })
        .then(take)
        .catch((error: unknown) => {
          console.error("Could not fit the reference's window", error);
        });
    };
    const observer = new ResizeObserver(fit);
    observer.observe(island);
    let settle: number | undefined;
    const moved = host.onMoved(() => {
      window.clearTimeout(settle);
      settle = window.setTimeout(fit, SETTLE_MS);
    });
    const told = host.listen<ReferenceLayout>(LAYOUT_EVENT, (event) => {
      take(event.payload);
    });
    return () => {
      observer.disconnect();
      window.clearTimeout(settle);
      for (const unlistening of [moved, told]) {
        void unlistening.then((unlisten) => {
          unlisten();
        });
      }
    };
  }, [instantly]);

  return { islandRef, ...placed };
}
