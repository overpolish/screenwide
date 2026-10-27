// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMotionValue } from "motion/react";
import { useEffect, useRef } from "react";

import { ISLAND } from "../../../../primitives/island/island-geometry";
import { type IslandPart } from "../../../../primitives/island/island-outline-sync";

import { type IslandContent } from "./island-parts";

// Where the island stands, chosen so the pointer travels least to reach it:
// beside the toolbar button that opened a pen or a mode, and over the strip
// for anything chosen, there, by the keyboard or on the picture. An
// annotation chosen on the picture is still stood by on the strip, over its
// bar, so the island never covers the picture being worked on. Its place is
// read every frame, so it follows what it belongs to as the strip changes.

export type Spot = {
  kind: "strip" | "tool";
  /** What it stands by, in the editor window. */
  selector: string;
};

export const spotOf = (content: IslandContent): Spot =>
  content.from.kind === "tool"
    ? { kind: "tool", selector: `[data-island-tool="${content.from.tool}"]` }
    : { kind: "strip", selector: '[data-island-anchor="selection"]' };

/** Between the island and the workspace's sides, and what it stands by. */
const EDGE_PX = 12;
/** How quickly it settles where it belongs, in ms: exponential, so it slows
 * into place without passing it. */
const FOLLOW_MS = 90;

/** Moves the island's frame to its spot every frame, easing within a kind
 * of spot and coming out in place when it is shown. */
export function useIslandPlace(
  workspace: HTMLElement | null,
  spot: Spot | null,
  isShown: boolean,
) {
  const x = useMotionValue(0);
  const y = useMotionValue(0);
  const frameRef = useRef<HTMLDivElement>(null);
  // The island's parts as it last reported them, for its size.
  const partsRef = useRef<IslandPart[]>([]);
  const wantRef = useRef({ isShown, spot });
  useEffect(() => {
    wantRef.current = { isShown, spot };
  });

  useEffect(() => {
    if (!workspace) return;
    const root = workspace.closest("[data-editor-window]") ?? workspace;
    let frame = 0;
    let previous: number | null = null;
    const tick = (now: number) => {
      frame = requestAnimationFrame(tick);
      const { isShown: shown, spot: want } = wantRef.current;
      const box = workspace.getBoundingClientRect();
      const row = frameRef.current?.querySelector("[role='toolbar']");
      const frameBox = frameRef.current?.getBoundingClientRect();
      if (!shown || !want || !row || !frameBox) {
        // Hidden, it stays where it was, and comes out where it belongs.
        if (!shown) previous = null;
        return;
      }
      const parts = partsRef.current;
      const extent = (start: "x" | "y", size: "height" | "width") =>
        parts.length === 0
          ? 0
          : Math.max(...parts.map((part) => part[start] + part[size])) -
            Math.min(...parts.map((part) => part[start]));
      const width = extent("x", "width");
      const height = extent("y", "height");
      const rowBox = row.getBoundingClientRect();
      const rowHeight = rowBox.height + ISLAND.rowInset * 2;
      // How far the shape's top sits below the frame's, whatever the frame's
      // own offset, so the shape is placed rather than the frame.
      const shapeTop = rowBox.top - ISLAND.rowInset - frameBox.top;

      const over = root.querySelector(want.selector)?.getBoundingClientRect();
      let middle = box.width / 2;
      let top = box.height - EDGE_PX - height;
      if (over && want.kind === "tool") {
        middle = EDGE_PX + width / 2;
        top = over.top + over.height / 2 - box.top - rowHeight / 2;
      } else if (over) {
        middle = over.left + over.width / 2 - box.left;
      }
      const half = width / 2 + EDGE_PX;
      const targetX =
        box.width < half * 2
          ? box.width / 2
          : Math.min(Math.max(middle, half), box.width - half);
      const targetY = Math.min(
        Math.max(top, EDGE_PX),
        box.height - EDGE_PX - height,
      );
      const ease =
        previous === null ? 1 : 1 - Math.exp(-(now - previous) / FOLLOW_MS);
      previous = now;
      x.set(x.get() + (targetX - x.get()) * ease);
      const toY = targetY - shapeTop;
      y.set(y.get() + (toY - y.get()) * ease);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [workspace, x, y]);
  return { frameRef, partsRef, x, y };
}
